//! Onboarding Wizard Service
//! Complete implementation per specification

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    BusinessContextAnswers, BusinessIdeaAnswers,
    onboarding_wizard::{WizardBusinessIdea, OnboardingAnswer},
    CompleteOnboardingResponse, CountryResponse, CountrySelectionAnswers,
    FieldOption, FieldValidation, Founder, FounderInvitationResponse, FounderResponse,
    FounderTypeAnswers, InviteCofounderRequest, ResumeOnboardingResponse, SaveStepAnswersRequest,
    SaveStepResponse, StartOnboardingResponse, StepContent, StepField, SupportedCountry,
    ValidationError,
};
use crate::utils::{AppError, Result};

pub struct OnboardingWizardService {
    db: PgPool,
}

impl OnboardingWizardService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    // ============================================
    // 1. START ONBOARDING SESSION
    // ============================================
    pub async fn start_onboarding(
        &self,
        user_id: Uuid,
    ) -> Result<StartOnboardingResponse> {
        // Check for existing active session
        let existing = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT id FROM onboarding_sessions 
            WHERE user_id = $1 AND status = 'active'
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        let session_id = if let Some(id) = existing {
            // Resume existing
            sqlx::query(
                r#"
                UPDATE onboarding_sessions 
                SET resumed_at = NOW(), resume_count = resume_count + 1
                WHERE id = $1
                "#,
            )
            .bind(id)
            .execute(&self.db)
            .await?;
            id
        } else {
            // Create new session
            sqlx::query_scalar::<_, Uuid>(
                r#"
                INSERT INTO onboarding_sessions (
                    user_id, current_step, progress_percentage, status, data,
                    wizard_started_at, last_completed_step
                )
                VALUES ($1, 'step_1', 0, 'active', '{}', NOW(), 0)
                RETURNING id
                "#,
            )
            .bind(user_id)
            .fetch_one(&self.db)
            .await?
        };

        // Track analytics event
        sqlx::query(
            r#"
            INSERT INTO onboarding_analytics (session_id, user_id, event_type, event_data)
            VALUES ($1, $2, 'wizard_started', '{}')
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        let first_step = self.get_step_content(1).await?;

        Ok(StartOnboardingResponse {
            session_id,
            current_step: 1,
            progress_percentage: 0,
            first_step,
        })
    }

    // ============================================
    // 2. SAVE STEP ANSWERS
    // ============================================
    pub async fn save_step_answers(
        &self,
        user_id: Uuid,
        req: SaveStepAnswersRequest,
    ) -> Result<SaveStepResponse> {
        // Verify session belongs to user
        let _session = sqlx::query_as::<_, crate::models::OnboardingSession>(
            "SELECT * FROM onboarding_sessions WHERE id = $1 AND user_id = $2"
        )
        .bind(req.session_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Session not found".to_string()))?;

        let validation_errors = self.validate_step_answers(&req).await?;
        if !validation_errors.is_empty() {
            return Ok(SaveStepResponse {
                success: false,
                next_step: None,
                progress_percentage: self.calculate_progress(req.step),
                next_step_content: None,
                validation_errors: Some(validation_errors),
            });
        }

        // Save answers based on step
        match &req.answers {
            crate::models::StepAnswers::Step1(answers) => {
                self.save_country_selection(user_id, req.session_id, req.step, answers).await?;
            }
            crate::models::StepAnswers::Step2(answers) => {
                self.save_founder_type(user_id, req.session_id, req.step, answers).await?;
            }
            crate::models::StepAnswers::Step3(answers) => {
                self.save_business_idea(user_id, req.session_id, req.step, answers).await?;
            }
            crate::models::StepAnswers::Step4(answers) => {
                self.save_business_context(user_id, req.session_id, req.step, answers).await?;
            }
            crate::models::StepAnswers::Step5(answers) => {
                self.save_review_confirmation(user_id, req.session_id, req.step, answers).await?;
            }
        }

        // Update session progress
        let next_step = if req.step < 5 { req.step + 1 } else { 5 };
        let progress = self.calculate_progress(next_step);

        sqlx::query(
            r#"
            UPDATE onboarding_sessions 
            SET current_step = $1, progress_percentage = $2, last_completed_step = $3, updated_at = NOW()
            WHERE id = $4
            "#,
        )
        .bind(format!("step_{}", next_step))
        .bind(progress)
        .bind(req.step)
        .bind(req.session_id)
        .execute(&self.db)
        .await?;

        // Track analytics
        sqlx::query(
            r#"
            INSERT INTO onboarding_analytics (session_id, user_id, event_type, step_number, event_data)
            VALUES ($1, $2, 'step_completed', $3, '{}')
            "#,
        )
        .bind(req.session_id)
        .bind(user_id)
        .bind(req.step)
        .execute(&self.db)
        .await?;

        let next_step_content = if next_step <= 5 {
            Some(self.get_step_content(next_step).await?)
        } else {
            None
        };

        Ok(SaveStepResponse {
            success: true,
            next_step: if next_step <= 5 { Some(next_step) } else { None },
            progress_percentage: progress,
            next_step_content,
            validation_errors: None,
        })
    }

    // ============================================
    // 3. GET STEP CONTENT
    // ============================================
    pub async fn get_step_content(&self, step: i32) -> Result<StepContent> {
        match step {
            1 => Ok(self.get_step_1_content()),
            2 => Ok(self.get_step_2_content()),
            3 => Ok(self.get_step_3_content()),
            4 => Ok(self.get_step_4_content()),
            5 => Ok(self.get_step_5_content()),
            _ => Err(AppError::Validation(format!("Invalid step number: {}", step))),
        }
    }

    fn get_step_1_content(&self) -> StepContent {
        StepContent {
            step_number: 1,
            title: "Where will your business be based?".to_string(),
            description: "This helps us show you the right compliance requirements and available services.".to_string(),
            helper_text: Some("Your choice determines currency, compliance rules, and available partners.".to_string()),
            fields: vec![
                StepField {
                    key: "country".to_string(),
                    label: "Primary Country".to_string(),
                    field_type: "select".to_string(),
                    required: true,
                    placeholder: Some("Select your country...".to_string()),
                    options: None, // Fetched dynamically
                    validation: Some(FieldValidation {
                        min_length: Some(2),
                        max_length: Some(2),
                        min_value: None,
                        max_value: None,
                        pattern: None,
                    }),
                    tooltip: Some("This is where your business will be legally registered.".to_string()),
                },
                StepField {
                    key: "secondary_countries".to_string(),
                    label: "Additional Countries (Optional)".to_string(),
                    field_type: "multiselect".to_string(),
                    required: false,
                    placeholder: Some("Select additional operating countries...".to_string()),
                    options: None,
                    validation: Some(FieldValidation {
                        min_length: None,
                        max_length: Some(5),
                        min_value: None,
                        max_value: None,
                        pattern: None,
                    }),
                    tooltip: Some("For businesses operating across multiple African countries.".to_string()),
                },
                StepField {
                    key: "has_physical_presence".to_string(),
                    label: "Physical Presence".to_string(),
                    field_type: "radio".to_string(),
                    required: false,
                    placeholder: None,
                    options: Some(vec![
                        FieldOption { value: "physical".to_string(), label: "Physical office/location".to_string(), description: Some("We have or plan to have a physical location".to_string()), icon: Some("building".to_string()) },
                        FieldOption { value: "digital".to_string(), label: "Digital-only".to_string(), description: Some("Fully online business with no physical location".to_string()), icon: Some("computer".to_string()) },
                    ]),
                    validation: None,
                    tooltip: Some("Helps us understand your operational needs.".to_string()),
                },
            ],
        }
    }

    fn get_step_2_content(&self) -> StepContent {
        StepContent {
            step_number: 2,
            title: "How are you building this?".to_string(),
            description: "Tell us about your founding team setup.".to_string(),
            helper_text: Some("This determines collaboration features and equity tools we'll set up for you.".to_string()),
            fields: vec![
                StepField {
                    key: "founder_type".to_string(),
                    label: "Founder Type".to_string(),
                    field_type: "radio".to_string(),
                    required: true,
                    placeholder: None,
                    options: Some(vec![
                        FieldOption { value: "solo".to_string(), label: "Solo Founder".to_string(), description: Some("I'm building this alone—for now. We'll help you build systems that scale with you.".to_string()), icon: Some("user".to_string()) },
                        FieldOption { value: "team".to_string(), label: "Team".to_string(), description: Some("I have co-founders or team members. We'll help you set up roles, equity, and team management from day one.".to_string()), icon: Some("users".to_string()) },
                    ]),
                    validation: None,
                    tooltip: Some("You can always add co-founders later.".to_string()),
                },
            ],
        }
    }

    fn get_step_3_content(&self) -> StepContent {
        StepContent {
            step_number: 3,
            title: "Tell us about your business idea".to_string(),
            description: "What problem are you solving? Who are you solving it for?".to_string(),
            helper_text: Some("Be specific—the more detail you give, the better our AI can help you.".to_string()),
            fields: vec![
                StepField {
                    key: "business_idea".to_string(),
                    label: "Your Business Idea".to_string(),
                    field_type: "textarea".to_string(),
                    required: true,
                    placeholder: Some("I want to build an app that helps small farmers in Nigeria connect directly with buyers...".to_string()),
                    options: None,
                    validation: Some(FieldValidation {
                        min_length: Some(50),
                        max_length: Some(5000),
                        min_value: None,
                        max_value: None,
                        pattern: None,
                    }),
                    tooltip: Some("Minimum 50 characters. Tell us the problem, your solution, and who it's for.".to_string()),
                },
            ],
        }
    }

    fn get_step_4_content(&self) -> StepContent {
        StepContent {
            step_number: 4,
            title: "Tell us more (Optional)".to_string(),
            description: "These questions help us give you better recommendations.".to_string(),
            helper_text: Some("You can skip any of these and fill them in later.".to_string()),
            fields: vec![
                StepField {
                    key: "target_customers".to_string(),
                    label: "Who will use your product?".to_string(),
                    field_type: "radio".to_string(),
                    required: false,
                    placeholder: None,
                    options: Some(vec![
                        FieldOption { value: "b2c".to_string(), label: "Consumers (B2C)".to_string(), description: None, icon: None },
                        FieldOption { value: "b2b".to_string(), label: "Businesses (B2B)".to_string(), description: None, icon: None },
                        FieldOption { value: "both".to_string(), label: "Both".to_string(), description: None, icon: None },
                        FieldOption { value: "b2g".to_string(), label: "Government (B2G)".to_string(), description: None, icon: None },
                    ]),
                    validation: None,
                    tooltip: Some("Your primary customer segment.".to_string()),
                },
                StepField {
                    key: "revenue_model".to_string(),
                    label: "How do you plan to make money?".to_string(),
                    field_type: "multiselect".to_string(),
                    required: false,
                    placeholder: Some("Select revenue models...".to_string()),
                    options: Some(vec![
                        FieldOption { value: "subscription".to_string(), label: "Subscription".to_string(), description: Some("Recurring monthly/yearly payments".to_string()), icon: None },
                        FieldOption { value: "transaction".to_string(), label: "Transaction fees".to_string(), description: Some("Take a cut of each transaction".to_string()), icon: None },
                        FieldOption { value: "advertising".to_string(), label: "Advertising".to_string(), description: Some("Sell ad space to third parties".to_string()), icon: None },
                        FieldOption { value: "marketplace".to_string(), label: "Marketplace commission".to_string(), description: Some("Commission on sales through platform".to_string()), icon: None },
                        FieldOption { value: "freemium".to_string(), label: "Freemium".to_string(), description: Some("Free basic version, paid premium features".to_string()), icon: None },
                        FieldOption { value: "unsure".to_string(), label: "Not sure yet".to_string(), description: Some("We'll help you figure this out".to_string()), icon: None },
                    ]),
                    validation: None,
                    tooltip: Some("You can select multiple options.".to_string()),
                },
                StepField {
                    key: "current_stage".to_string(),
                    label: "Where are you right now?".to_string(),
                    field_type: "radio".to_string(),
                    required: false,
                    placeholder: None,
                    options: Some(vec![
                        FieldOption { value: "idea".to_string(), label: "Just an idea".to_string(), description: Some("Exploring the concept".to_string()), icon: None },
                        FieldOption { value: "mvp".to_string(), label: "Building MVP".to_string(), description: Some("Actively building the product".to_string()), icon: None },
                        FieldOption { value: "launched".to_string(), label: "Launched".to_string(), description: Some("Product is live with users".to_string()), icon: None },
                        FieldOption { value: "growing".to_string(), label: "Growing".to_string(), description: Some("Scaling up operations".to_string()), icon: None },
                        FieldOption { value: "revenue".to_string(), label: "Generating revenue".to_string(), description: Some("Making money".to_string()), icon: None },
                    ]),
                    validation: None,
                    tooltip: Some("Your current startup stage.".to_string()),
                },
                StepField {
                    key: "industry".to_string(),
                    label: "What industry are you in?".to_string(),
                    field_type: "multiselect".to_string(),
                    required: false,
                    placeholder: Some("Select industries...".to_string()),
                    options: Some(vec![
                        FieldOption { value: "fintech".to_string(), label: "Fintech".to_string(), description: None, icon: None },
                        FieldOption { value: "agriculture".to_string(), label: "Agriculture".to_string(), description: None, icon: None },
                        FieldOption { value: "healthtech".to_string(), label: "Healthtech".to_string(), description: None, icon: None },
                        FieldOption { value: "edtech".to_string(), label: "Edtech".to_string(), description: None, icon: None },
                        FieldOption { value: "ecommerce".to_string(), label: "E-commerce".to_string(), description: None, icon: None },
                        FieldOption { value: "saas".to_string(), label: "SaaS".to_string(), description: None, icon: None },
                        FieldOption { value: "logistics".to_string(), label: "Logistics".to_string(), description: None, icon: None },
                        FieldOption { value: "entertainment".to_string(), label: "Entertainment".to_string(), description: None, icon: None },
                        FieldOption { value: "other".to_string(), label: "Other".to_string(), description: None, icon: None },
                    ]),
                    validation: None,
                    tooltip: Some("Select all that apply.".to_string()),
                },
                StepField {
                    key: "funding_status".to_string(),
                    label: "Have you raised money?".to_string(),
                    field_type: "radio".to_string(),
                    required: false,
                    placeholder: None,
                    options: Some(vec![
                        FieldOption { value: "bootstrapped".to_string(), label: "Bootstrapped".to_string(), description: Some("Self-funded".to_string()), icon: None },
                        FieldOption { value: "friends_family".to_string(), label: "Friends and family".to_string(), description: None, icon: None },
                        FieldOption { value: "angel".to_string(), label: "Angel investment".to_string(), description: None, icon: None },
                        FieldOption { value: "preseed".to_string(), label: "Pre-seed".to_string(), description: None, icon: None },
                        FieldOption { value: "seed".to_string(), label: "Seed".to_string(), description: None, icon: None },
                        FieldOption { value: "series_a".to_string(), label: "Series A+".to_string(), description: None, icon: None },
                    ]),
                    validation: None,
                    tooltip: Some("Your current funding stage.".to_string()),
                },
            ],
        }
    }

    fn get_step_5_content(&self) -> StepContent {
        StepContent {
            step_number: 5,
            title: "Review & Confirm".to_string(),
            description: "Here's what you've told us. Ready to generate your startup blueprint?".to_string(),
            helper_text: Some("Our AI will analyze your answers and create a personalized startup plan. This takes about 30 seconds.".to_string()),
            fields: vec![
                StepField {
                    key: "confirmed".to_string(),
                    label: "I'm ready".to_string(),
                    field_type: "checkbox".to_string(),
                    required: true,
                    placeholder: None,
                    options: Some(vec![
                        FieldOption { value: "true".to_string(), label: "Yes, generate my startup blueprint".to_string(), description: Some("I confirm the information above is accurate.".to_string()), icon: None },
                    ]),
                    validation: None,
                    tooltip: None,
                },
                StepField {
                    key: "terms_accepted".to_string(),
                    label: "Terms".to_string(),
                    field_type: "checkbox".to_string(),
                    required: true,
                    placeholder: None,
                    options: Some(vec![
                        FieldOption { value: "true".to_string(), label: "I agree to the Terms of Service and Privacy Policy".to_string(), description: None, icon: None },
                    ]),
                    validation: None,
                    tooltip: None,
                },
            ],
        }
    }

    // ============================================
    // 4. SAVE SPECIFIC STEP DATA
    // ============================================
    async fn save_country_selection(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        step: i32,
        answers: &CountrySelectionAnswers,
    ) -> Result<()> {
        let answer_json = json!({
            "country": answers.country,
            "secondary_countries": answers.secondary_countries,
            "has_physical_presence": answers.has_physical_presence,
            "is_digital_only": answers.is_digital_only,
        });

        sqlx::query(
            r#"
            INSERT INTO onboarding_answers (user_id, session_id, step_number, question_key, answer_value, answer_json, completed_at)
            VALUES ($1, $2, $3, 'country_selection', $4, $5, NOW())
            ON CONFLICT (user_id, session_id, question_key) DO UPDATE SET
                answer_value = EXCLUDED.answer_value,
                answer_json = EXCLUDED.answer_json,
                completed_at = NOW(),
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(session_id)
        .bind(step)
        .bind(&answers.country)
        .bind(&answer_json)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn save_founder_type(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        step: i32,
        answers: &FounderTypeAnswers,
    ) -> Result<()> {
        let answer_json = json!({
            "founder_type": answers.founder_type,
            "team_size": answers.team_size,
            "cofounders": answers.cofounders,
        });

        sqlx::query(
            r#"
            INSERT INTO onboarding_answers (user_id, session_id, step_number, question_key, answer_value, answer_json, completed_at)
            VALUES ($1, $2, $3, 'founder_type', $4, $5, NOW())
            ON CONFLICT (user_id, session_id, question_key) DO UPDATE SET
                answer_value = EXCLUDED.answer_value,
                answer_json = EXCLUDED.answer_json,
                completed_at = NOW(),
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(session_id)
        .bind(step)
        .bind(&answers.founder_type)
        .bind(&answer_json)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn save_business_idea(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        step: i32,
        answers: &BusinessIdeaAnswers,
    ) -> Result<()> {
        let answer_json = json!({
            "business_idea": answers.business_idea,
        });

        // Save to onboarding_answers
        sqlx::query(
            r#"
            INSERT INTO onboarding_answers (user_id, session_id, step_number, question_key, answer_value, answer_json, completed_at)
            VALUES ($1, $2, $3, 'business_idea', $4, $5, NOW())
            ON CONFLICT (user_id, session_id, question_key) DO UPDATE SET
                answer_value = EXCLUDED.answer_value,
                answer_json = EXCLUDED.answer_json,
                completed_at = NOW(),
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(session_id)
        .bind(step)
        .bind(&answers.business_idea)
        .bind(&answer_json)
        .execute(&self.db)
        .await?;

        // Also create business_idea record with processing
        let processed_text = self.preprocess_idea_text(&answers.business_idea);
        let language = self.detect_language(&answers.business_idea);
        let keywords = self.extract_keywords(&answers.business_idea);
        let flagged = self.check_if_flagged(&answers.business_idea);

        sqlx::query(
            r#"
            INSERT INTO business_ideas (
                user_id, session_id, raw_idea_text, processed_idea_text,
                language_detected, keywords, flagged_for_review, flag_reason,
                is_active, version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, 1)
            "#,
        )
        .bind(user_id)
        .bind(session_id)
        .bind(&answers.business_idea)
        .bind(&processed_text)
        .bind(&language)
        .bind(&keywords)
        .bind(flagged.0)
        .bind(flagged.1)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn save_business_context(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        step: i32,
        answers: &BusinessContextAnswers,
    ) -> Result<()> {
        let answer_json = json!({
            "target_customers": answers.target_customers,
            "b2b_segment": answers.b2b_segment,
            "revenue_model": answers.revenue_model,
            "current_stage": answers.current_stage,
            "industry": answers.industry,
            "funding_status": answers.funding_status,
            "funding_amount": answers.funding_amount,
        });

        sqlx::query(
            r#"
            INSERT INTO onboarding_answers (user_id, session_id, step_number, question_key, answer_json, completed_at)
            VALUES ($1, $2, $3, 'business_context', $4, NOW())
            ON CONFLICT (user_id, session_id, question_key) DO UPDATE SET
                answer_json = EXCLUDED.answer_json,
                completed_at = NOW(),
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(session_id)
        .bind(step)
        .bind(&answer_json)
        .execute(&self.db)
        .await?;

        // Update business_idea with context
        if let Some(ref industries) = answers.industry {
            let industry_str = industries.join(",");
            sqlx::query(
                r#"
                UPDATE business_ideas 
                SET industry = $1, updated_at = NOW()
                WHERE session_id = $2 AND user_id = $3 AND is_active = true
                "#,
            )
            .bind(&industry_str)
            .bind(session_id)
            .bind(user_id)
            .execute(&self.db)
            .await?;
        }

        if let Some(ref revenue_models) = answers.revenue_model {
            sqlx::query(
                r#"
                UPDATE business_ideas 
                SET revenue_model = $1, updated_at = NOW()
                WHERE session_id = $2 AND user_id = $3 AND is_active = true
                "#,
            )
            .bind(json!(revenue_models))
            .bind(session_id)
            .bind(user_id)
            .execute(&self.db)
            .await?;
        }

        Ok(())
    }

    async fn save_review_confirmation(
        &self,
        _user_id: Uuid,
        _session_id: Uuid,
        _step: i32,
        _answers: &crate::models::ReviewAnswers,
    ) -> Result<()> {
        // Review step just confirms - no separate data to save
        Ok(())
    }

    // ============================================
    // 5. GET COUNTRIES
    // ============================================
    pub async fn get_supported_countries(&self) -> Result<Vec<CountryResponse>> {
        let countries = sqlx::query_as::<_, SupportedCountry>(
            r#"
            SELECT * FROM supported_countries 
            WHERE is_active = true 
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(countries.into_iter().map(Into::into).collect())
    }

    // ============================================
    // 6. INVITE CO-FOUNDER
    // ============================================
    pub async fn invite_cofounder(
        &self,
        user_id: Uuid,
        startup_id: Option<Uuid>,
        req: InviteCofounderRequest,
    ) -> Result<FounderInvitationResponse> {
        // Generate invitation token
        let token = format!("inv_{}", Uuid::new_v4().to_string().replace("-", ""));
        
        let equity_value = req.equity;

        let founder_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO founders (
                startup_id, invited_by, email, full_name, role, 
                equity_percentage, status, invitation_token, invitation_sent_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'invited', $7, NOW())
            RETURNING id
            "#,
        )
        .bind(startup_id)
        .bind(user_id)
        .bind(&req.email)
        .bind(&req.full_name)
        .bind(&req.role)
        .bind(equity_value)
        .bind(&token)
        .fetch_one(&self.db)
        .await?;

        Ok(FounderInvitationResponse {
            founder_id,
            email: req.email,
            status: "invited".to_string(),
            invitation_token: token.clone(),
            invitation_url: format!("/join/{}?token={}", startup_id.map(|s| s.to_string()).unwrap_or_default(), token),
        })
    }

    pub async fn get_cofounders_for_startup(
        &self,
        startup_id: Uuid,
    ) -> Result<Vec<FounderResponse>> {
        let founders = sqlx::query_as::<_, Founder>(
            r#"
            SELECT * FROM founders 
            WHERE startup_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(startup_id)
        .fetch_all(&self.db)
        .await?;

        Ok(founders.into_iter().map(|f| FounderResponse {
            id: f.id,
            email: f.email,
            full_name: f.full_name,
            role: f.role,
            equity_percentage: f.equity_percentage,
            status: f.status,
            joined_at: f.accepted_at,
        }).collect())
    }

    // ============================================
    // 7. COMPLETE ONBOARDING
    // ============================================
    pub async fn complete_onboarding(
        &self,
        user_id: Uuid,
        session_id: Uuid,
    ) -> Result<CompleteOnboardingResponse> {
        // Get session data
        let _session = sqlx::query_as::<_, crate::models::OnboardingSession>(
            "SELECT * FROM onboarding_sessions WHERE id = $1 AND user_id = $2"
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        // Get all answers
        let answers = sqlx::query_as::<_, OnboardingAnswer>(
            r#"
            SELECT * FROM onboarding_answers 
            WHERE session_id = $1 AND user_id = $2
            ORDER BY step_number ASC
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        // Get business idea
        let idea = sqlx::query_as::<_, WizardBusinessIdea>(
            r#"
            SELECT * FROM business_ideas 
            WHERE session_id = $1 AND user_id = $2 AND is_active = true
            ORDER BY version DESC
            LIMIT 1
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        // Extract country from answers
        let country_code = answers
            .iter()
            .find(|a| a.question_key == "country_selection")
            .and_then(|a| a.answer_value.clone())
            .unwrap_or_else(|| "ZA".to_string());

        // Extract founder type
        let _founder_type = answers
            .iter()
            .find(|a| a.question_key == "founder_type")
            .and_then(|a| a.answer_value.clone())
            .unwrap_or_else(|| "solo".to_string());

        // Extract industry
        let industry = idea
            .as_ref()
            .and_then(|i| i.industry.clone())
            .unwrap_or_else(|| "Technology".to_string());

        // Generate business name from idea
        let business_name = self.generate_business_name(idea.as_ref().map(|i| i.raw_idea_text.clone()).as_deref())
            .await?;

        let slug = slug::slugify(&business_name);

        // Create startup/business
        let startup_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO businesses (
                owner_id, name, slug, industry, country_code, 
                status, stage, description
            )
            VALUES ($1, $2, $3, $4, $5, 'active', 'idea', $6)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(&business_name)
        .bind(&slug)
        .bind(&industry)
        .bind(&country_code)
        .bind(idea.as_ref().map(|i| i.raw_idea_text.clone()))
        .fetch_one(&self.db)
        .await?;

        // Update business_idea with startup_id
        sqlx::query(
            "UPDATE business_ideas SET startup_id = $1 WHERE session_id = $2 AND user_id = $3"
        )
        .bind(startup_id)
        .bind(session_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        // Update onboarding_answers with startup_id
        sqlx::query(
            "UPDATE onboarding_answers SET startup_id = $1 WHERE session_id = $2 AND user_id = $3"
        )
        .bind(startup_id)
        .bind(session_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        // Update cofounders with startup_id (those invited during onboarding)
        sqlx::query(
            "UPDATE founders SET startup_id = $1 WHERE invited_by = $2 AND startup_id IS NULL"
        )
        .bind(startup_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        // Mark session as completed
        sqlx::query(
            r#"
            UPDATE onboarding_sessions 
            SET status = 'completed', 
                progress_percentage = 100,
                wizard_completed_at = NOW(),
                total_time_seconds = EXTRACT(EPOCH FROM (NOW() - wizard_started_at))::INTEGER
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .execute(&self.db)
        .await?;

        // Update user onboarding status
        sqlx::query(
            "UPDATE users SET onboarding_completed = true, onboarding_step = 'completed' WHERE id = $1"
        )
        .bind(user_id)
        .execute(&self.db)
        .await?;

        // Track completion
        sqlx::query(
            r#"
            INSERT INTO onboarding_analytics (session_id, user_id, event_type, event_data)
            VALUES ($1, $2, 'wizard_completed', '{}')
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(CompleteOnboardingResponse {
            startup_id,
            startup_name: business_name,
            dashboard_url: format!("/dashboard/startup/{}", startup_id),
            processing_status: "processing".to_string(),
            estimated_completion_seconds: 30,
        })
    }

    // ============================================
    // 8. RESUME ONBOARDING
    // ============================================
    pub async fn resume_onboarding(
        &self,
        user_id: Uuid,
        session_id: Uuid,
    ) -> Result<ResumeOnboardingResponse> {
        let session = sqlx::query_as::<_, crate::models::OnboardingSession>(
            "SELECT * FROM onboarding_sessions WHERE id = $1 AND user_id = $2"
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        // Get all saved answers
        let answers: serde_json::Value = sqlx::query_scalar(
            r#"
            SELECT COALESCE(json_object_agg(question_key, answer_json), '{}'::json)
            FROM onboarding_answers
            WHERE session_id = $1 AND user_id = $2
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        let last_completed = session
            .data
            .get("last_completed_step")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;

        let current_step = last_completed + 1;

        Ok(ResumeOnboardingResponse {
            session_id,
            last_completed_step: last_completed,
            current_step: current_step.min(5),
            progress_percentage: session.progress_percentage,
            saved_answers: answers,
            welcome_back_message: format!("Welcome back! You're {}% done. Let's finish setting up your startup.", session.progress_percentage),
        })
    }

    // ============================================
    // 9. HELPER METHODS
    // ============================================
    async fn validate_step_answers(&self, req: &SaveStepAnswersRequest) -> Result<Vec<ValidationError>> {
        let mut errors = vec![];

        match &req.answers {
            crate::models::StepAnswers::Step1(a) => {
                if a.country.is_empty() {
                    errors.push(ValidationError {
                        field: "country".to_string(),
                        message: "Please select a country".to_string(),
                    });
                }
                // Validate country exists
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM supported_countries WHERE code = $1 AND is_active = true)"
                )
                .bind(&a.country)
                .fetch_one(&self.db)
                .await?;

                if !exists {
                    errors.push(ValidationError {
                        field: "country".to_string(),
                        message: "Selected country is not supported".to_string(),
                    });
                }

                if let Some(ref secondary) = a.secondary_countries {
                    if secondary.len() > 5 {
                        errors.push(ValidationError {
                            field: "secondary_countries".to_string(),
                            message: "Maximum 5 secondary countries allowed".to_string(),
                        });
                    }
                }
            }
            crate::models::StepAnswers::Step2(a) => {
                if a.founder_type != "solo" && a.founder_type != "team" {
                    errors.push(ValidationError {
                        field: "founder_type".to_string(),
                        message: "Please select Solo or Team".to_string(),
                    });
                }

                if a.founder_type == "team" {
                    if let Some(ref cofounders) = a.cofounders {
                        let total_equity: f64 = cofounders.iter().map(|c| c.equity_percentage).sum();
                        if total_equity > 100.0 {
                            errors.push(ValidationError {
                                field: "cofounders".to_string(),
                                message: "Total equity cannot exceed 100%".to_string(),
                            });
                        }
                    }
                }
            }
            crate::models::StepAnswers::Step3(a) => {
                if a.business_idea.len() < 50 {
                    errors.push(ValidationError {
                        field: "business_idea".to_string(),
                        message: "Hmm, that's a bit short. Tell us more about your big idea! (minimum 50 characters)".to_string(),
                    });
                }
                if a.business_idea.len() > 5000 {
                    errors.push(ValidationError {
                        field: "business_idea".to_string(),
                        message: "That's quite detailed! Please keep it under 5000 characters.".to_string(),
                    });
                }
                // Check for gibberish/spam
                if self.is_gibberish(&a.business_idea) {
                    errors.push(ValidationError {
                        field: "business_idea".to_string(),
                        message: "Please enter a real business idea with actual words.".to_string(),
                    });
                }
            }
            _ => {}
        }

        Ok(errors)
    }

    fn calculate_progress(&self, step: i32) -> i32 {
        match step {
            1 => 0,
            2 => 25,
            3 => 50,
            4 => 75,
            5 => 90,
            _ => 100,
        }
    }

    fn preprocess_idea_text(&self, text: &str) -> String {
        text.trim()
            .replace("\r\n", "\n")
            .replace("\r", "\n")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn detect_language(&self, text: &str) -> Option<String> {
        // Simplified language detection - would use a library in production
        if text.chars().any(|c| matches!(c, 'à'|'è'|'ì'|'ò'|'ù'|'á'|'é'|'í'|'ó'|'ú'|'ñ')) {
            Some("es".to_string())
        } else if text.chars().any(|c| matches!(c, 'ç'|'é'|'à'|'è'|'ù'|'â'|'ê'|'î'|'ô'|'û')) {
            Some("fr".to_string())
        } else {
            Some("en".to_string())
        }
    }

    fn extract_keywords(&self, text: &str) -> serde_json::Value {
        // Simplified keyword extraction
        let common_words = ["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by"];
        let words: Vec<String> = text
            .to_lowercase()
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|w| w.len() > 3 && !common_words.contains(&w.to_lowercase().as_str()))
            .map(|w| w.to_string())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .take(10)
            .collect();
        
        json!(words)
    }

    fn check_if_flagged(&self, text: &str) -> (bool, Option<String>) {
        // Check for spam indicators
        let link_count = text.matches("http").count();
        if link_count > 3 {
            return (true, Some("Contains too many links".to_string()));
        }
        
        // Check for repeated characters
        let repeated = text.chars().collect::<Vec<_>>().windows(5).any(|w| {
            w.iter().all(|&c| c == w[0] && !c.is_whitespace())
        });
        if repeated {
            return (true, Some("Contains repeated characters".to_string()));
        }

        (false, None)
    }

    fn is_gibberish(&self, text: &str) -> bool {
        // Check if text looks like gibberish
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() < 3 {
            return true;
        }
        
        // Check for long strings without spaces (likely gibberish)
        if text.len() > 50 && !text.contains(' ') {
            return true;
        }

        // Check vowel ratio (gibberish usually has poor vowel distribution)
        let vowels = text.matches(|c: char| "aeiouAEIOU".contains(c)).count();
        let consonants = text.chars().filter(|c| c.is_alphabetic() && !"aeiouAEIOU".contains(*c)).count();
        
        if consonants > 0 {
            let ratio = vowels as f64 / consonants as f64;
            if ratio < 0.1 || ratio > 2.0 {
                return true;
            }
        }

        false
    }

    async fn generate_business_name(&self, idea_text: Option<&str>) -> Result<String> {
        if let Some(text) = idea_text {
            // Extract key words and create a name
            let words: Vec<&str> = text
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .take(3)
                .collect();
            
            if !words.is_empty() {
                let name = words.join(" ");
                return Ok(format!("{} Ventures", name));
            }
        }
        
        Ok("My Startup".to_string())
    }
}


