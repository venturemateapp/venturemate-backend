//! AI Startup Engine Service
//! Complete implementation per specification

use chrono::Utc;
use f64;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    AiConfidence, AiValidationLog, BusinessAnalysisPrompt, BusinessAnalysisResponse,
    BusinessIdentity, BusinessModel, CacheQueryResult, ComplianceRequirements,
    CreateCacheEntryRequest, CreateGenerationLogRequest, CreateValidationLogRequest,
    EnhancementPrompt, EnhancementResponse, GenerationLog, GenerationMetadata,
    GenerationStatusResponse, IndustryClassificationCache, IndustryDefinition,
    MarketIntelligence, OnboardingData, ProcessStartupRequest, ProcessStartupResponse,
    RegenerateFieldRequest, RegistrationRequirement, RegulatoryRequirement, RevenueModel,
    StartupBlueprint, SubIndustryDefinition, UpdateGenerationLogRequest,
    FALLBACK_INDUSTRIES, FALLBACK_REVENUE_MODELS,
};
use crate::services::AIService;
use crate::utils::{AppError, Result};

pub struct AiStartupEngineService {
    db: PgPool,
    ai_service: AIService,
}

impl AiStartupEngineService {
    pub fn new(db: PgPool, ai_service: AIService) -> Self {
        Self { db, ai_service }
    }

    // ============================================================================
    // MAIN PROCESSING PIPELINE
    // ============================================================================

    /// Process startup through AI engine
    pub async fn process_startup(
        &self,
        user_id: Uuid,
        req: ProcessStartupRequest,
    ) -> Result<ProcessStartupResponse> {
        let start_time = Utc::now();

        // Step 1: Input validation
        self.validate_input(&req.onboarding_data).await?;

        // Step 2: Create generation log
        let input_data = serde_json::to_value(&req.onboarding_data)
            .map_err(|e| AppError::Internal(format!("Failed to serialize input: {}", e)))?;
        
        let generation_id = self
            .create_generation_log(CreateGenerationLogRequest {
                user_id,
                business_id: req.business_id,
                input_data,
                onboarding_session_id: None,
                ai_model: "claude-3-opus-20240229".to_string(),
            })
            .await?;

        // Step 3: Check cache for similar classifications
        let cache_result = self
            .check_classification_cache(&req.onboarding_data.business_idea)
            .await?;

        // Step 4: Run AI business analysis (in background or sync)
        let onboarding_data = req.onboarding_data.clone();
        let ai_service = self.ai_service.clone();
        let db = self.db.clone();

        // For now, process synchronously with timeout handling
        match self
            .run_ai_pipeline(generation_id, &onboarding_data, cache_result)
            .await
        {
            Ok(_) => {
                let elapsed = Utc::now().signed_duration_since(start_time).num_milliseconds();
                
                Ok(ProcessStartupResponse {
                    generation_id,
                    status: "completed".to_string(),
                    estimated_time: (elapsed / 1000) as i32,
                    message: "Startup blueprint generated successfully".to_string(),
                })
            }
            Err(e) => {
                // Log error and return partial response
                self.update_generation_log(
                    generation_id,
                    UpdateGenerationLogRequest {
                        status: Some("failed".to_string()),
                        error_message: Some(e.to_string()),
                        ..Default::default()
                    },
                )
                .await?;

                Err(e)
            }
        }
    }

    /// Run the complete AI pipeline
    async fn run_ai_pipeline(
        &self,
        generation_id: Uuid,
        onboarding_data: &OnboardingData,
        cache_result: CacheQueryResult,
    ) -> Result<()> {
        let pipeline_start = Utc::now();

        // Step 1: AI Business Analysis (Prompt 1)
        let analysis = match self
            .run_business_analysis(generation_id, onboarding_data, cache_result)
            .await
        {
            Ok(analysis) => analysis,
            Err(e) => {
                tracing::warn!("AI analysis failed, using fallback: {}", e);
                self.fallback_analysis(onboarding_data).await?
            }
        };

        // Step 2: Industry Classification Verification
        let verified_industry = self
            .verify_industry_classification(&analysis.industry, &analysis.sub_industry)
            .await?;

        // Step 3: Revenue Model Suggestion
        let revenue_models = self
            .suggest_revenue_models(&verified_industry, &analysis.suggested_revenue_models)
            .await?;

        // Step 4: Regulatory Rules Engine
        let compliance = self
            .generate_compliance_requirements(&onboarding_data.country, &verified_industry)
            .await?;

        // Step 5: AI Enhancement (Prompt 2) - Optional, don't fail if this fails
        let enhancement = match self
            .run_enhancement(&analysis.business_name, &verified_industry, &analysis.value_proposition)
            .await
        {
            Ok(enhancement) => Some(enhancement),
            Err(e) => {
                tracing::warn!("Enhancement failed: {}", e);
                None
            }
        };

        // Step 6: Assemble Blueprint
        let blueprint = self
            .assemble_blueprint(&analysis, enhancement, revenue_models, compliance)
            .await?;

        // Calculate confidence scores
        let confidence = AiConfidence {
            overall_score: analysis.confidence_score,
            industry_classification: analysis.confidence_score * 0.95,
            revenue_model: analysis.confidence_score * 0.85,
            business_name: analysis.confidence_score * 0.90,
        };

        let processing_time = Utc::now()
            .signed_duration_since(pipeline_start)
            .num_milliseconds() as i32;

        // Step 7: Save results
        self.update_generation_log(
            generation_id,
            UpdateGenerationLogRequest {
                status: Some("completed".to_string()),
                parsed_output: Some(serde_json::to_value(&analysis)
                    .map_err(|e| AppError::Internal(format!("Failed to serialize analysis: {}", e)))?),
                blueprint: Some(serde_json::to_value(&blueprint)
                    .map_err(|e| AppError::Internal(format!("Failed to serialize blueprint: {}", e)))?),
                processing_time_ms: Some(processing_time),
                confidence_overall: Some(confidence.overall_score as f64),
                confidence_industry: Some(confidence.industry_classification as f64),
                confidence_revenue: Some(confidence.revenue_model as f64),
                confidence_name: Some(confidence.business_name as f64),
                ..Default::default()
            },
        )
        .await?;

        // Cache successful classification
        if confidence.industry_classification > 0.7 {
            self.cache_classification(
                &onboarding_data.business_idea,
                &verified_industry,
                analysis.confidence_score,
            )
            .await?;
        }

        Ok(())
    }

    // ============================================================================
    // AI PROMPT EXECUTION
    // ============================================================================

    /// Run business analysis prompt (Prompt 1)
    async fn run_business_analysis(
        &self,
        generation_id: Uuid,
        data: &OnboardingData,
        cache_result: CacheQueryResult,
    ) -> Result<BusinessAnalysisResponse> {
        let prompt = self.build_analysis_prompt(data, cache_result);

        // Update log with prompt
        self.update_generation_log(
            generation_id,
            UpdateGenerationLogRequest {
                prompt_sent: Some(prompt.clone()),
                ..Default::default()
            },
        )
        .await?;

        // Call AI service
        let system_prompt = "You are VentureMate's AI Startup Analyst. Analyze business ideas and return structured JSON responses. Always respond with valid JSON only.";
        
        let ai_response = self
            .ai_service
            .generate_text(system_prompt, &prompt, 4000, Some(0.7))
            .await?;

        // Update log with raw response
        self.update_generation_log(
            generation_id,
            UpdateGenerationLogRequest {
                raw_ai_response: Some(ai_response.clone()),
                ..Default::default()
            },
        )
        .await?;

        // Parse JSON response
        let parsed: BusinessAnalysisResponse = self.parse_ai_response(&ai_response)?;

        Ok(parsed)
    }

    /// Run enhancement prompt (Prompt 2)
    async fn run_enhancement(
        &self,
        business_name: &str,
        industry: &str,
        value_proposition: &str,
    ) -> Result<EnhancementResponse> {
        let prompt = format!(
            r#"Based on the following business analysis, generate enhanced content:

BUSINESS NAME: {}
INDUSTRY: {}
VALUE PROPOSITION: {}

Return JSON with:
{{
    "tagline": "A catchy 5-8 word tagline",
    "elevator_pitch": "30-second pitch suitable for investors",
    "mission_statement": "One sentence mission",
    "vision_statement": "Where will this be in 5 years?",
    "key_metrics": ["3-5 KPIs this business should track"],
    "risk_factors": ["Top risks and mitigation strategies"],
    "growth_strategy": "How to acquire first 100 customers",
    "team_needs": ["What expertise is needed"]
}}"#,
            business_name, industry, value_proposition
        );

        let system_prompt = "You are a startup strategist. Generate compelling business content. Respond with valid JSON only.";
        
        let ai_response = self
            .ai_service
            .generate_text(system_prompt, &prompt, 2000, Some(0.7))
            .await?;

        let parsed: EnhancementResponse = self.parse_ai_response(&ai_response)?;
        Ok(parsed)
    }

    // ============================================================================
    // PROMPT BUILDERS
    // ============================================================================

    fn build_analysis_prompt(&self, data: &OnboardingData, cache_result: CacheQueryResult) -> String {
        let cache_hint = if cache_result.found {
            format!("\nHINT: This idea may be related to {} industry.", cache_result.industry.unwrap_or_default())
        } else {
            String::new()
        };

        let optional_context = data.optional_context.as_ref();
        let target_customers = optional_context
            .and_then(|c| c.target_customers.as_ref())
            .map(|s| format!("\nTARGET CUSTOMERS: {}", s))
            .unwrap_or_default();

        format!(
            r#"You are VentureMate's AI Startup Analyst. Analyze the following business idea and return a structured analysis.

BUSINESS IDEA: {}{}
COUNTRY: {}
FOUNDER TYPE: {}{}

Return a JSON object with:
{{
    "business_name": "Best business name",
    "alternative_names": ["2-4 alternative names"],
    "industry": "One of: Fintech, Agritech, Healthtech, Edtech, E-commerce, SaaS, Logistics, Marketplace, Media, CleanTech, PropTech, Other",
    "sub_industry": "More specific classification",
    "value_proposition": "One sentence core value",
    "problem_statement": "What problem does this solve? (1-2 sentences)",
    "solution_description": "How does it work? (1-2 sentences)",
    "target_customers": "B2B, B2C, B2B2C, etc.",
    "target_customer_description": "Detailed ideal customer description",
    "suggested_revenue_models": ["2-3 revenue models"],
    "market_size_estimate": "TAM/SAM/SOM estimate",
    "competitive_advantage": "What makes this unique?",
    "key_challenges": ["Top 3 challenges"],
    "suggested_next_steps": ["3 immediate actions"],
    "confidence_score": 0.85
}}

Rules:
- Be specific to African markets when relevant
- Names should be professional and memorable
- If idea is vague, make reasonable assumptions
- Confidence score between 0 and 1"#,
            data.business_idea,
            cache_hint,
            data.country,
            data.founder_type,
            target_customers
        )
    }

    // ============================================================================
    // INDUSTRY CLASSIFICATION
    // ============================================================================

    /// Verify and normalize industry classification
    async fn verify_industry_classification(
        &self,
        ai_industry: &str,
        ai_sub_industry: &Option<String>,
    ) -> Result<String> {
        // Check if industry is in predefined list
        let normalized = ai_industry.trim().to_lowercase();
        
        let valid_industry = FALLBACK_INDUSTRIES
            .iter()
            .find(|&&ind| ind.to_lowercase() == normalized)
            .map(|&s| s.to_string())
            .unwrap_or_else(|| "Other".to_string());

        // Log validation if AI returned invalid industry
        if valid_industry != ai_industry {
            tracing::warn!(
                "AI returned invalid industry '{}', normalized to '{}'",
                ai_industry,
                valid_industry
            );
        }

        Ok(valid_industry)
    }

    /// Check cache for similar business ideas
    async fn check_classification_cache(
        &self,
        business_idea: &str,
    ) -> Result<CacheQueryResult> {
        // Extract keywords (simple implementation - first 50 chars)
        let keywords = business_idea.to_lowercase();
        let keywords_hash = self.hash_keywords(&keywords);

        let cached: Option<IndustryClassificationCache> = sqlx::query_as(
            "SELECT * FROM industry_classification_cache WHERE keywords_hash = $1"
        )
        .bind(&keywords_hash)
        .fetch_optional(&self.db)
        .await?;

        if let Some(entry) = cached {
            // Update usage count
            sqlx::query(
                "UPDATE industry_classification_cache SET usage_count = usage_count + 1, last_used_at = NOW() WHERE id = $1"
            )
            .bind(entry.id)
            .execute(&self.db)
            .await?;

            return Ok(CacheQueryResult {
                found: true,
                industry: Some(entry.industry),
                sub_industry: entry.sub_industry,
                confidence: Some(entry.confidence_score.to_string().parse().unwrap_or(0.0)),
            });
        }

        Ok(CacheQueryResult {
            found: false,
            industry: None,
            sub_industry: None,
            confidence: None,
        })
    }

    /// Cache successful classification
    async fn cache_classification(
        &self,
        business_idea: &str,
        industry: &str,
        confidence: f32,
    ) -> Result<()> {
        let keywords = business_idea.to_lowercase();
        let keywords_hash = self.hash_keywords(&keywords);

        sqlx::query(
            r#"
            INSERT INTO industry_classification_cache 
                (keywords_hash, keywords_text, industry, confidence_score)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (keywords_hash) DO UPDATE SET
                usage_count = industry_classification_cache.usage_count + 1,
                last_used_at = NOW()
            "#
        )
        .bind(&keywords_hash)
        .bind(&keywords[..keywords.len().min(500)]) // Limit text length
        .bind(industry)
        .bind(confidence as f64)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    fn hash_keywords(&self, keywords: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(keywords.as_bytes());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    // ============================================================================
    // REVENUE MODEL SUGGESTIONS
    // ============================================================================

    /// Suggest revenue models based on industry
    async fn suggest_revenue_models(
        &self,
        industry: &str,
        ai_suggestions: &[String],
    ) -> Result<Vec<RevenueModel>> {
        // Get industry defaults from database
        let industry_def: Option<IndustryDefinition> = sqlx::query_as(
            "SELECT * FROM industry_definitions WHERE industry_code = $1 AND is_active = true"
        )
        .bind(industry.to_lowercase())
        .fetch_optional(&self.db)
        .await?;

        let mut models = Vec::new();

        // Use AI suggestions first
        for (i, model) in ai_suggestions.iter().take(3).enumerate() {
            models.push(RevenueModel {
                model: model.clone(),
                description: if i == 0 {
                    "Primary revenue model".to_string()
                } else {
                    "Secondary revenue stream".to_string()
                },
            });
        }

        // Fill with industry defaults if needed
        if models.is_empty() {
            if let Some(def) = industry_def {
                if let Some(primary) = def.primary_revenue_models.as_array() {
                    for (i, model) in primary.iter().take(2).enumerate() {
                        if let Some(name) = model.as_str() {
                            models.push(RevenueModel {
                                model: name.to_string(),
                                description: if i == 0 {
                                    "Industry-standard primary model".to_string()
                                } else {
                                    "Common secondary model".to_string()
                                },
                            });
                        }
                    }
                }
            }
        }

        // Ensure at least one model
        if models.is_empty() {
            models.push(RevenueModel {
                model: "Transaction Fee".to_string(),
                description: "Charge a fee per transaction".to_string(),
            });
        }

        Ok(models)
    }

    // ============================================================================
    // REGULATORY COMPLIANCE
    // ============================================================================

    /// Generate compliance requirements based on country and industry
    async fn generate_compliance_requirements(
        &self,
        country: &str,
        industry: &str,
    ) -> Result<ComplianceRequirements> {
        let country_code = self.normalize_country_code(country);

        // Query regulatory requirements
        let requirements: Vec<RegulatoryRequirement> = sqlx::query_as(
            r#"
            SELECT * FROM regulatory_requirements
            WHERE country_code = $1
            AND is_active = true
            AND (applicable_industries @> $2::jsonb OR applicable_industries @> '["all"]'::jsonb)
            ORDER BY priority ASC
            "#
        )
        .bind(&country_code)
        .bind(json!([industry.to_lowercase()]))
        .fetch_all(&self.db)
        .await?;

        let mut registrations = Vec::new();
        let mut total_timeline = 0;
        let mut total_cost = 0.0;

        for req in requirements {
            let docs: Vec<String> = req
                .required_documents
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            registrations.push(RegistrationRequirement {
                name: req.requirement_name,
                authority: req.authority_name.unwrap_or_default(),
                timeline_days: req.estimated_time_days.unwrap_or(0),
                cost_estimate: req.estimated_cost_min.unwrap_or_default(),
                priority: req.priority,
                documents_required: docs,
                condition: req.condition_note,
            });

            total_timeline += req.estimated_time_days.unwrap_or(0);
            total_cost += req.estimated_cost_min.unwrap_or_default();
        }

        Ok(ComplianceRequirements {
            country: country.to_string(),
            registrations,
            total_estimated_timeline: total_timeline,
            total_estimated_cost: total_cost,
        })
    }

    fn normalize_country_code(&self, country: &str) -> String {
        match country.to_uppercase().as_str() {
            "NIGERIA" | "NG" => "NG".to_string(),
            "KENYA" | "KE" => "KE".to_string(),
            "SOUTH AFRICA" | "ZA" | "SA" => "ZA".to_string(),
            "GHANA" | "GH" => "GH".to_string(),
            "RWANDA" | "RW" => "RW".to_string(),
            _ => country[..2.min(country.len())].to_uppercase(),
        }
    }

    // ============================================================================
    // BLUEPRINT ASSEMBLY
    // ============================================================================

    /// Assemble all components into final blueprint
    async fn assemble_blueprint(
        &self,
        analysis: &BusinessAnalysisResponse,
        enhancement: Option<EnhancementResponse>,
        revenue_models: Vec<RevenueModel>,
        compliance: ComplianceRequirements,
    ) -> Result<StartupBlueprint> {
        let primary_model = revenue_models
            .first()
            .cloned()
            .unwrap_or_else(|| RevenueModel {
                model: "Transaction Fee".to_string(),
                description: "Standard model".to_string(),
            });

        let secondary_models = if revenue_models.len() > 1 {
            revenue_models[1..].to_vec()
        } else {
            Vec::new()
        };

        Ok(StartupBlueprint {
            business_identity: BusinessIdentity {
                business_name: analysis.business_name.clone(),
                alternative_names: analysis.alternative_names.clone(),
                tagline: enhancement.as_ref().map(|e| e.tagline.clone()).unwrap_or_default(),
                elevator_pitch: enhancement
                    .as_ref()
                    .map(|e| e.elevator_pitch.clone())
                    .unwrap_or_default(),
                mission_statement: enhancement
                    .as_ref()
                    .map(|e| e.mission_statement.clone())
                    .unwrap_or_default(),
                vision_statement: enhancement
                    .as_ref()
                    .map(|e| e.vision_statement.clone())
                    .unwrap_or_default(),
            },
            market_intelligence: MarketIntelligence {
                industry: analysis.industry.clone(),
                sub_industry: analysis.sub_industry.clone(),
                value_proposition: analysis.value_proposition.clone(),
                problem_statement: analysis.problem_statement.clone(),
                solution_description: analysis.solution_description.clone(),
                target_customers: analysis.target_customers.clone(),
                target_customer_description: analysis.target_customer_description.clone(),
                market_size_estimate: analysis.market_size_estimate.clone(),
                competitive_advantage: analysis.competitive_advantage.clone(),
                key_challenges: analysis.key_challenges.clone(),
            },
            business_model: BusinessModel {
                primary_revenue_model: primary_model.model,
                primary_model_description: primary_model.description,
                secondary_revenue_models: secondary_models,
                pricing_suggestions: None,
            },
            compliance_requirements: compliance,
            ai_confidence: AiConfidence {
                overall_score: analysis.confidence_score,
                industry_classification: analysis.confidence_score * 0.95,
                revenue_model: analysis.confidence_score * 0.85,
                business_name: analysis.confidence_score * 0.90,
            },
            suggested_next_steps: analysis.suggested_next_steps.clone(),
            generation_metadata: GenerationMetadata {
                model_used: "claude-3-opus-20240229".to_string(),
                processing_time_ms: 0, // Will be updated
                tokens_used: 0,        // Will be updated
                generated_at: Utc::now(),
            },
        })
    }

    // ============================================================================
    // FALLBACK SYSTEM
    // ============================================================================

    /// Generate fallback analysis when AI fails
    async fn fallback_analysis(&self, data: &OnboardingData) -> Result<BusinessAnalysisResponse> {
        let keywords: Vec<&str> = data.business_idea.split_whitespace().collect();
        let name = if !keywords.is_empty() {
            format!("{}Tech Solutions", keywords[0])
        } else {
            "Startup Solutions".to_string()
        };

        Ok(BusinessAnalysisResponse {
            business_name: name.clone(),
            alternative_names: vec![format!("{} Africa", name), "New Venture".to_string()],
            industry: "Other".to_string(),
            sub_industry: None,
            value_proposition: "Providing innovative solutions to customer needs".to_string(),
            problem_statement: "Customers face challenges that need better solutions".to_string(),
            solution_description: "A technology platform addressing these challenges".to_string(),
            target_customers: "B2C".to_string(),
            target_customer_description: "General consumers seeking better solutions".to_string(),
            suggested_revenue_models: vec!["Subscription".to_string(), "Transaction Fee".to_string()],
            market_size_estimate: "TAM: $1B".to_string(),
            competitive_advantage: "First-mover advantage in target market".to_string(),
            key_challenges: vec![
                "Market penetration".to_string(),
                "Customer acquisition".to_string(),
                "Funding".to_string(),
            ],
            suggested_next_steps: vec![
                "Validate problem with potential customers".to_string(),
                "Build MVP".to_string(),
                "Register business".to_string(),
            ],
            confidence_score: 0.5,
        })
    }

    // ============================================================================
    // RESPONSE PARSING
    // ============================================================================

    /// Parse AI response with error handling
    fn parse_ai_response<T: serde::de::DeserializeOwned>(&self, response: &str) -> Result<T> {
        // Try to find JSON in the response
        let json_str = if response.trim().starts_with('{') {
            response.trim()
        } else {
            // Try to extract JSON from text
            let start = response.find('{');
            let end = response.rfind('}');
            
            match (start, end) {
                (Some(s), Some(e)) if s < e => &response[s..=e],
                _ => response.trim(),
            }
        };

        serde_json::from_str(json_str).map_err(|e| {
            AppError::BadRequest(format!("Failed to parse AI response: {}", e))
        })
    }

    // ============================================================================
    // INPUT VALIDATION
    // ============================================================================

    async fn validate_input(&self, data: &OnboardingData) -> Result<()> {
        if data.business_idea.len() < 10 {
            return Err(AppError::Validation(
                "Business idea must be at least 10 characters".to_string()
            ));
        }

        if data.business_idea.len() > 5000 {
            return Err(AppError::Validation(
                "Business idea must not exceed 5000 characters".to_string()
            ));
        }

        if data.country.is_empty() {
            return Err(AppError::Validation("Country is required".to_string()));
        }

        Ok(())
    }

    // ============================================================================
    // DATABASE OPERATIONS
    // ============================================================================

    async fn create_generation_log(&self, req: CreateGenerationLogRequest) -> Result<Uuid> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO generation_logs 
                (id, user_id, business_id, input_data, onboarding_session_id, ai_model, status)
            VALUES ($1, $2, $3, $4, $5, $6, 'processing')
            "#
        )
        .bind(id)
        .bind(req.user_id)
        .bind(req.business_id)
        .bind(req.input_data)
        .bind(req.onboarding_session_id)
        .bind(req.ai_model)
        .execute(&self.db)
        .await?;

        Ok(id)
    }

    async fn update_generation_log(
        &self,
        id: Uuid,
        req: UpdateGenerationLogRequest,
    ) -> Result<()> {
        if let Some(status) = req.status {
            sqlx::query("UPDATE generation_logs SET status = $1 WHERE id = $2")
                .bind(&status)
                .bind(id)
                .execute(&self.db)
                .await?;
            
            if status == "completed" || status == "failed" {
                sqlx::query("UPDATE generation_logs SET completed_at = NOW() WHERE id = $1")
                    .bind(id)
                    .execute(&self.db)
                    .await?;
            }
        }

        if let Some(prompt) = req.prompt_sent {
            sqlx::query("UPDATE generation_logs SET prompt_sent = $1 WHERE id = $2")
                .bind(prompt)
                .bind(id)
                .execute(&self.db)
                .await?;
        }

        if let Some(response) = req.raw_ai_response {
            sqlx::query("UPDATE generation_logs SET raw_ai_response = $1 WHERE id = $2")
                .bind(response)
                .bind(id)
                .execute(&self.db)
                .await?;
        }

        if let Some(parsed) = req.parsed_output {
            sqlx::query("UPDATE generation_logs SET parsed_output = $1 WHERE id = $2")
                .bind(parsed)
                .bind(id)
                .execute(&self.db)
                .await?;
        }

        if let Some(blueprint) = req.blueprint {
            sqlx::query("UPDATE generation_logs SET blueprint = $1 WHERE id = $2")
                .bind(blueprint)
                .bind(id)
                .execute(&self.db)
                .await?;
        }

        if let Some(time) = req.processing_time_ms {
            sqlx::query("UPDATE generation_logs SET processing_time_ms = $1 WHERE id = $2")
                .bind(time)
                .bind(id)
                .execute(&self.db)
                .await?;
        }

        if let Some(tokens) = req.input_tokens {
            sqlx::query("UPDATE generation_logs SET input_tokens = $1 WHERE id = $2")
                .bind(tokens)
                .bind(id)
                .execute(&self.db)
                .await?;
        }

        if let Some(tokens) = req.output_tokens {
            sqlx::query("UPDATE generation_logs SET output_tokens = $1 WHERE id = $2")
                .bind(tokens)
                .bind(id)
                .execute(&self.db)
                .await?;
        }

        if let Some(cost) = req.estimated_cost {
            sqlx::query("UPDATE generation_logs SET estimated_cost = $1 WHERE id = $2")
                .bind(cost)
                .bind(id)
                .execute(&self.db)
                .await?;
        }

        if let Some(error) = req.error_message {
            sqlx::query("UPDATE generation_logs SET error_message = $1 WHERE id = $2")
                .bind(error)
                .bind(id)
                .execute(&self.db)
                .await?;
        }

        // Update confidence scores
        if let Some(score) = req.confidence_overall {
            sqlx::query("UPDATE generation_logs SET confidence_overall = $1 WHERE id = $2")
                .bind(score)
                .bind(id)
                .execute(&self.db)
                .await?;
        }

        if let Some(score) = req.confidence_industry {
            sqlx::query("UPDATE generation_logs SET confidence_industry = $1 WHERE id = $2")
                .bind(score)
                .bind(id)
                .execute(&self.db)
                .await?;
        }

        if let Some(score) = req.confidence_revenue {
            sqlx::query("UPDATE generation_logs SET confidence_revenue = $1 WHERE id = $2")
                .bind(score)
                .bind(id)
                .execute(&self.db)
                .await?;
        }

        if let Some(score) = req.confidence_name {
            sqlx::query("UPDATE generation_logs SET confidence_name = $1 WHERE id = $2")
                .bind(score)
                .bind(id)
                .execute(&self.db)
                .await?;
        }

        Ok(())
    }

    // ============================================================================
    // PUBLIC API METHODS
    // ============================================================================

    /// Get generation status
    pub async fn get_generation_status(&self, generation_id: Uuid) -> Result<GenerationStatusResponse> {
        let log: GenerationLog = sqlx::query_as(
            "SELECT * FROM generation_logs WHERE id = $1"
        )
        .bind(generation_id)
        .fetch_one(&self.db)
        .await?;

        let blueprint = log.blueprint.map(|v| {
            serde_json::from_value(v).unwrap_or_else(|_| StartupBlueprint {
                business_identity: BusinessIdentity {
                    business_name: "Unknown".to_string(),
                    alternative_names: vec![],
                    tagline: "".to_string(),
                    elevator_pitch: "".to_string(),
                    mission_statement: "".to_string(),
                    vision_statement: "".to_string(),
                },
                market_intelligence: MarketIntelligence {
                    industry: "Other".to_string(),
                    sub_industry: None,
                    value_proposition: "".to_string(),
                    problem_statement: "".to_string(),
                    solution_description: "".to_string(),
                    target_customers: "".to_string(),
                    target_customer_description: "".to_string(),
                    market_size_estimate: "".to_string(),
                    competitive_advantage: "".to_string(),
                    key_challenges: vec![],
                },
                business_model: BusinessModel {
                    primary_revenue_model: "".to_string(),
                    primary_model_description: "".to_string(),
                    secondary_revenue_models: vec![],
                    pricing_suggestions: None,
                },
                compliance_requirements: ComplianceRequirements {
                    country: "".to_string(),
                    registrations: vec![],
                    total_estimated_timeline: 0,
                    total_estimated_cost: 0.0,
                },
                ai_confidence: AiConfidence {
                    overall_score: 0.0,
                    industry_classification: 0.0,
                    revenue_model: 0.0,
                    business_name: 0.0,
                },
                suggested_next_steps: vec![],
                generation_metadata: GenerationMetadata {
                    model_used: "".to_string(),
                    processing_time_ms: 0,
                    tokens_used: 0,
                    generated_at: Utc::now(),
                },
            })
        });

        Ok(GenerationStatusResponse {
            generation_id: log.id,
            status: log.status,
            blueprint,
            created_at: log.created_at,
            completed_at: log.completed_at,
        })
    }

    /// Regenerate specific field
    pub async fn regenerate_field(
        &self,
        _user_id: Uuid,
        req: RegenerateFieldRequest,
    ) -> Result<Value> {
        // Get existing blueprint
        let blueprint: Option<Value> = sqlx::query_scalar(
            "SELECT blueprint FROM generation_logs WHERE business_id = $1 ORDER BY created_at DESC LIMIT 1"
        )
        .bind(req.startup_id)
        .fetch_optional(&self.db)
        .await?;

        if blueprint.is_none() {
            return Err(AppError::NotFound("No blueprint found for this startup".to_string()));
        }

        // Build context-aware prompt for regeneration
        let context = req.context.unwrap_or_default();
        let prompt = format!(
            "Based on the existing startup, generate a new {}. Context: {}\n\nReturn only the new {} as a JSON string value.",
            req.field, context, req.field
        );

        let system_prompt = "You are a startup naming and branding expert. Generate creative, professional suggestions.";
        
        let ai_response = self
            .ai_service
            .generate_text(system_prompt, &prompt, 500, Some(0.8))
            .await?;

        // Parse response
        let new_value: Value = serde_json::from_str(&ai_response)
            .unwrap_or_else(|_| Value::String(ai_response.trim().to_string()));

        // Log the regeneration
        let generation_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM generation_logs WHERE business_id = $1 ORDER BY created_at DESC LIMIT 1"
        )
        .bind(req.startup_id)
        .fetch_one(&self.db)
        .await?;

        sqlx::query(
            "INSERT INTO ai_validation_logs (generation_log_id, field_name, corrected_value, action_taken) VALUES ($1, $2, $3, 'user_regenerated')"
        )
        .bind(generation_id)
        .bind(&req.field)
        .bind(&new_value.to_string())
        .execute(&self.db)
        .await?;

        Ok(new_value)
    }

    /// List industries
    pub async fn list_industries(&self) -> Result<Vec<IndustryDefinition>> {
        let industries: Vec<IndustryDefinition> = sqlx::query_as(
            "SELECT * FROM industry_definitions WHERE is_active = true ORDER BY industry_name"
        )
        .fetch_all(&self.db)
        .await?;

        Ok(industries)
    }

    /// List regulatory requirements for a country
    pub async fn get_regulatory_requirements(
        &self,
        country_code: &str,
        industry: Option<&str>,
    ) -> Result<Vec<RegulatoryRequirement>> {
        let requirements: Vec<RegulatoryRequirement> = if let Some(ind) = industry {
            sqlx::query_as(
                r#"
                SELECT * FROM regulatory_requirements
                WHERE country_code = $1
                AND is_active = true
                AND (applicable_industries @> $2::jsonb OR applicable_industries @> '["all"]'::jsonb)
                ORDER BY priority ASC
                "#
            )
            .bind(country_code.to_uppercase())
            .bind(json!([ind.to_lowercase()]))
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as(
                "SELECT * FROM regulatory_requirements WHERE country_code = $1 AND is_active = true ORDER BY priority"
            )
            .bind(country_code.to_uppercase())
            .fetch_all(&self.db)
            .await?
        };

        Ok(requirements)
    }
}

// Default implementations for request structs
impl Default for UpdateGenerationLogRequest {
    fn default() -> Self {
        Self {
            status: None,
            raw_ai_response: None,
            parsed_output: None,
            processing_time_ms: None,
            input_tokens: None,
            output_tokens: None,
            estimated_cost: None,
            error_message: None,
            blueprint: None,
            confidence_overall: None,
            confidence_industry: None,
            confidence_revenue: None,
            confidence_name: None,
            prompt_sent: None,
        }
    }
}
