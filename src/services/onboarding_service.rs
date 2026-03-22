use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    AiAnalysis, BusinessDetailsRequest, FounderProfileRequest,
    IdeaIntakeRequest, IdeaIntakeResponse, OnboardingCompleteResponse, OnboardingSession,
    OnboardingSessionResponse, OnboardingStatusResponse, ReviewOnboardingRequest,
};
use crate::utils::{AppError, Result};

pub struct OnboardingService {
    db: PgPool,
}

impl OnboardingService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Start a new onboarding session
    pub async fn start_session(&self, user_id: Uuid) -> Result<OnboardingSessionResponse> {
        let session = sqlx::query_as::<_, OnboardingSession>(
            r#"
            INSERT INTO onboarding_sessions (user_id, current_step, progress_percentage, status, data)
            VALUES ($1, 'idea_intake', 0, 'active', '{}')
            RETURNING *
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(session.into())
    }

    /// Submit idea intake
    pub async fn submit_idea_intake(
        &self,
        user_id: Uuid,
        req: IdeaIntakeRequest,
    ) -> Result<IdeaIntakeResponse> {
        // Verify session belongs to user
        let _session = sqlx::query_as::<_, OnboardingSession>(
            "SELECT * FROM onboarding_sessions WHERE id = $1 AND user_id = $2"
        )
        .bind(req.session_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Session not found".to_string()))?;

        // Store idea data
        let data = json!({
            "business_idea": req.business_idea,
            "problem_statement": req.problem_statement,
            "target_customers": req.target_customers,
            "country_code": req.country_code,
            "city": req.city,
            "founder_type": req.founder_type,
            "team_size": req.team_size,
            "has_cofounder": req.has_cofounder,
        });

        sqlx::query(
            r#"
            UPDATE onboarding_sessions 
            SET current_step = 'founder_profile', 
                progress_percentage = 25,
                data = data || $1::jsonb,
                updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(&data)
        .bind(req.session_id)
        .execute(&self.db)
        .await?;

        // Generate AI analysis (mock for now)
        let analysis = AiAnalysis {
            industry: "Technology".to_string(),
            sub_industry: "SaaS".to_string(),
            market_size: "$10B Global Market".to_string(),
            complexity: "medium".to_string(),
            estimated_launch_time: "6-8 weeks".to_string(),
            suggested_business_models: vec![
                "Subscription".to_string(),
                "Freemium".to_string(),
                "Enterprise Licensing".to_string(),
            ],
            viability_score: Some(8),
        };

        Ok(IdeaIntakeResponse {
            session_id: req.session_id,
            ai_analysis: analysis,
            next_step: "founder_profile".to_string(),
            progress_percentage: 25,
        })
    }

    /// Submit founder profile
    pub async fn submit_founder_profile(
        &self,
        user_id: Uuid,
        req: FounderProfileRequest,
    ) -> Result<OnboardingSessionResponse> {
        let data = json!({
            "experience_level": req.experience_level,
            "background": req.background,
            "skills": req.skills,
            "availability": req.availability,
            "funding_preference": req.funding_preference,
            "motivation": req.motivation,
            "challenges": req.challenges,
        });

        let session = sqlx::query_as::<_, OnboardingSession>(
            r#"
            UPDATE onboarding_sessions 
            SET current_step = 'business_details', 
                progress_percentage = 50,
                data = data || $1::jsonb,
                updated_at = NOW()
            WHERE id = $2 AND user_id = $3
            RETURNING *
            "#,
        )
        .bind(&data)
        .bind(req.session_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(session.into())
    }

    /// Submit business details
    pub async fn submit_business_details(
        &self,
        user_id: Uuid,
        req: BusinessDetailsRequest,
    ) -> Result<OnboardingSessionResponse> {
        let data = json!({
            "preferred_business_name": req.preferred_business_name,
            "alternative_names": req.alternative_names,
            "business_model": req.business_model,
            "revenue_streams": req.revenue_streams,
            "initial_funding": req.initial_funding,
            "currency": req.currency,
            "timeline": req.timeline,
            "legal_structure_preference": req.legal_structure_preference,
        });

        let session = sqlx::query_as::<_, OnboardingSession>(
            r#"
            UPDATE onboarding_sessions 
            SET current_step = 'review', 
                progress_percentage = 75,
                data = data || $1::jsonb,
                updated_at = NOW()
            WHERE id = $2 AND user_id = $3
            RETURNING *
            "#,
        )
        .bind(&data)
        .bind(req.session_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(session.into())
    }

    /// Complete onboarding and create business
    pub async fn complete_onboarding(
        &self,
        user_id: Uuid,
        req: ReviewOnboardingRequest,
    ) -> Result<OnboardingCompleteResponse> {
        let session = sqlx::query_as::<_, OnboardingSession>(
            "SELECT * FROM onboarding_sessions WHERE id = $1 AND user_id = $2"
        )
        .bind(req.session_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        // Extract business name from session data
        let business_name = session
            .data
            .get("preferred_business_name")
            .and_then(|v| v.as_str())
            .unwrap_or("My Business")
            .to_string();

        let industry = session
            .data
            .get("industry")
            .and_then(|v| v.as_str())
            .unwrap_or("Technology")
            .to_string();

        let country_code = session
            .data
            .get("country_code")
            .and_then(|v| v.as_str())
            .unwrap_or("ZA")
            .to_string();

        // Generate slug
        let slug = slug::slugify(&business_name);

        // Create business
        let business_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO businesses (owner_id, name, slug, industry, country_code, status, stage)
            VALUES ($1, $2, $3, $4, $5, 'active', 'idea')
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(&business_name)
        .bind(&slug)
        .bind(&industry)
        .bind(&country_code)
        .fetch_one(&self.db)
        .await?;

        // Update user onboarding status
        sqlx::query(
            "UPDATE users SET onboarding_completed = true, onboarding_step = 'completed' WHERE id = $1"
        )
        .bind(user_id)
        .execute(&self.db)
        .await?;

        // Mark session as completed
        sqlx::query(
            "UPDATE onboarding_sessions SET status = 'completed', progress_percentage = 100 WHERE id = $1"
        )
        .bind(req.session_id)
        .execute(&self.db)
        .await?;

        // Create generation jobs if requested
        let mut generation_jobs = vec![];
        if req.generate_options.business_plan {
            let job_id = self.create_generation_job(business_id, user_id, "business_plan").await?;
            generation_jobs.push(crate::models::GenerationJobInfo {
                job_id,
                job_type: "business_plan".to_string(),
                status: "queued".to_string(),
            });
        }
        if req.generate_options.branding_kit {
            let job_id = self.create_generation_job(business_id, user_id, "branding_kit").await?;
            generation_jobs.push(crate::models::GenerationJobInfo {
                job_id,
                job_type: "branding_kit".to_string(),
                status: "queued".to_string(),
            });
        }
        if req.generate_options.website {
            let job_id = self.create_generation_job(business_id, user_id, "website").await?;
            generation_jobs.push(crate::models::GenerationJobInfo {
                job_id,
                job_type: "website".to_string(),
                status: "queued".to_string(),
            });
        }
        if req.generate_options.pitch_deck {
            let job_id = self.create_generation_job(business_id, user_id, "pitch_deck").await?;
            generation_jobs.push(crate::models::GenerationJobInfo {
                job_id,
                job_type: "pitch_deck".to_string(),
                status: "queued".to_string(),
            });
        }

        Ok(OnboardingCompleteResponse {
            business_id,
            generation_jobs,
            estimated_completion: Utc::now() + chrono::Duration::minutes(5),
            dashboard_url: format!("/dashboard/business/{}", business_id),
        })
    }

    /// Get onboarding status
    pub async fn get_status(&self, user_id: Uuid) -> Result<OnboardingStatusResponse> {
        let session = sqlx::query_as::<_, OnboardingSession>(
            "SELECT * FROM onboarding_sessions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1"
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        let user = sqlx::query_as::<_, crate::models::User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        let steps_completed = if user.onboarding_completed {
            vec![
                "idea_intake".to_string(),
                "founder_profile".to_string(),
                "business_details".to_string(),
                "review".to_string(),
            ]
        } else if let Some(ref s) = session {
            match s.current_step.as_str() {
                "idea_intake" => vec![],
                "founder_profile" => vec!["idea_intake".to_string()],
                "business_details" => vec!["idea_intake".to_string(), "founder_profile".to_string()],
                "review" => vec!["idea_intake".to_string(), "founder_profile".to_string(), "business_details".to_string()],
                _ => vec![],
            }
        } else {
            vec![]
        };

        let current_step = session.as_ref().map(|s| s.current_step.clone()).unwrap_or_else(|| "idea_intake".to_string());
        let overall_progress = if user.onboarding_completed { 
            100.0 
        } else { 
            session.map(|s| s.progress_percentage as f32).unwrap_or(0.0) 
        };
        
        Ok(OnboardingStatusResponse {
            user_id,
            steps_completed,
            current_step,
            overall_progress,
        })
    }

    async fn create_generation_job(
        &self,
        business_id: Uuid,
        user_id: Uuid,
        job_type: &str,
    ) -> Result<Uuid> {
        let job_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO ai_generation_jobs (business_id, user_id, job_type, status, progress)
            VALUES ($1, $2, $3, 'queued', 0)
            RETURNING id
            "#,
        )
        .bind(business_id)
        .bind(user_id)
        .bind(job_type)
        .fetch_one(&self.db)
        .await?;

        Ok(job_id)
    }
}
