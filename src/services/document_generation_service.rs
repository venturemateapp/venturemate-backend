//! Document Generation Service
//! 
//! Generates professional business plans, pitch decks, one-pagers,
//! and other investor-ready documents.

use crate::utils::{AppError, Result};
use crate::models::documents::{
    BusinessPlanContent, CompetitiveAnalysis, Competitor,
    CompanyOverview, MarketingSales, OperationsSection, ManagementTeam,
    DocumentStatusResponse, DocumentType, ExecutiveSummary, ExpenseCategory,
    Feature, FinancialProjections, FundingRequest,
    FundUse, GeneratedDocument, GeneratedDocumentResponse, GenerateBusinessPlanRequest,
    GenerateDocumentResponse, GeneratePitchDeckRequest, MarketAnalysis, MarketSize,
    Metric, PitchDeckContent, PitchDeckSlide, PricingTier, ProblemStatement,
    RevenueStream, SolutionSection, TargetSegment, TeamMember, YearProjection,
    get_pitch_deck_templates, BusinessModelSection, PitchDeckTemplate,
};
use crate::services::ai_service::AIService;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct DocumentGenerationService {
    db: PgPool,
    ai_service: Arc<AIService>,
}

impl DocumentGenerationService {
    pub fn new(db: PgPool, ai_service: Arc<AIService>) -> Self {
        Self { db, ai_service }
    }

    /// Generate a business plan
    pub async fn generate_business_plan(
        &self,
        user_id: Uuid,
        req: GenerateBusinessPlanRequest,
    ) -> Result<GenerateDocumentResponse> {
        info!("Generating business plan for business: {}", req.business_id);

        // Verify business exists and get details
        let business = self.get_business_details(req.business_id, user_id).await?;

        // Create document record
        let doc_id = Uuid::new_v4();
        let document_type = DocumentType::BusinessPlan;
        let document_name = format!("{} - Business Plan v1", business.name);

        sqlx::query(
            "INSERT INTO generated_documents (id, business_id, user_id, document_type, document_name, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, 'generating', NOW(), NOW())"
        )
        .bind(doc_id)
        .bind(req.business_id)
        .bind(user_id)
        .bind(document_type.as_str())
        .bind(&document_name)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        // Spawn background generation
        let db_clone = self.db.clone();
        let ai_service_clone = self.ai_service.clone();
        let business_clone = business;
        let years = req.years_projection.unwrap_or(3);
        let include_financials = req.include_financials;

        tokio::spawn(async move {
            if let Err(e) = Self::generate_business_plan_background(
                db_clone,
                ai_service_clone,
                doc_id,
                business_clone,
                years,
                include_financials,
            ).await {
                error!("Business plan generation failed: {}", e);
            }
        });

        Ok(GenerateDocumentResponse {
            generation_id: doc_id,
            status: "processing".to_string(),
            estimated_seconds: 45,
        })
    }

    /// Generate a pitch deck
    pub async fn generate_pitch_deck(
        &self,
        user_id: Uuid,
        req: GeneratePitchDeckRequest,
    ) -> Result<GenerateDocumentResponse> {
        info!("Generating pitch deck for business: {}", req.business_id);

        // Verify business exists and get details
        let business = self.get_business_details(req.business_id, user_id).await?;

        // Create document record
        let doc_id = Uuid::new_v4();
        let document_type = DocumentType::PitchDeck;
        let document_name = format!("{} - Pitch Deck v1", business.name);

        sqlx::query(
            "INSERT INTO generated_documents (id, business_id, user_id, document_type, document_name, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, 'generating', NOW(), NOW())"
        )
        .bind(doc_id)
        .bind(req.business_id)
        .bind(user_id)
        .bind(document_type.as_str())
        .bind(&document_name)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        // Spawn background generation
        let db_clone = self.db.clone();
        let ai_service_clone = self.ai_service.clone();
        let business_clone = business;
        let template = req.template;

        tokio::spawn(async move {
            if let Err(e) = Self::generate_pitch_deck_background(
                db_clone,
                ai_service_clone,
                doc_id,
                business_clone,
                template,
            ).await {
                error!("Pitch deck generation failed: {}", e);
            }
        });

        Ok(GenerateDocumentResponse {
            generation_id: doc_id,
            status: "processing".to_string(),
            estimated_seconds: 30,
        })
    }

    /// Get document status
    pub async fn get_document_status(
        &self,
        _user_id: Uuid,
        document_id: Uuid,
    ) -> Result<DocumentStatusResponse> {
        let doc: Option<GeneratedDocument> = sqlx::query_as(
            "SELECT * FROM generated_documents WHERE id = $1"
        )
        .bind(document_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        let Some(doc) = doc else {
            return Ok(DocumentStatusResponse {
                status: "not_found".to_string(),
                document: None,
                error_message: Some("Document not found".to_string()),
                progress_percent: None,
            });
        };

        let document_response = if doc.status == "ready" {
            Some(GeneratedDocumentResponse {
                id: doc.id,
                business_id: doc.business_id,
                document_type: doc.document_type.clone(),
                document_name: doc.document_name.unwrap_or_default(),
                file_format: doc.file_format.unwrap_or_default(),
                file_size: doc.file_size.unwrap_or(0),
                version: doc.version,
                template_used: doc.template_used,
                status: doc.status,
                download_count: doc.download_count,
                download_url: Some(format!("/api/v1/documents/{}/download", doc.id)),
                generated_at: doc.generated_at.unwrap_or(doc.created_at),
                expires_at: doc.expires_at,
            })
        } else {
            None
        };

        Ok(DocumentStatusResponse {
            status: doc.status,
            document: document_response,
            error_message: None,
            progress_percent: if doc.status == "generating" { Some(50) } else { None },
        })
    }

    /// Get documents for a business
    pub async fn get_business_documents(
        &self,
        user_id: Uuid,
        business_id: Uuid,
        doc_type: Option<DocumentType>,
    ) -> Result<Vec<GeneratedDocumentResponse>> {
        // Verify ownership
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM businesses WHERE id = $1 AND user_id = $2)"
        )
        .bind(business_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        if !exists {
            return Err(AppError::NotFound("Business not found".to_string()));
        }

        let docs: Vec<GeneratedDocument> = if let Some(t) = doc_type {
            sqlx::query_as(
                "SELECT * FROM generated_documents WHERE business_id = $1 AND document_type = $2 ORDER BY created_at DESC"
            )
            .bind(business_id)
            .bind(t.as_str())
            .fetch_all(&self.db)
            .await
            .map_err(AppError::Database)?
        } else {
            sqlx::query_as(
                "SELECT * FROM generated_documents WHERE business_id = $1 ORDER BY created_at DESC"
            )
            .bind(business_id)
            .fetch_all(&self.db)
            .await
            .map_err(AppError::Database)?
        };

        let responses = docs.into_iter()
            .map(|doc| GeneratedDocumentResponse {
                id: doc.id,
                business_id: doc.business_id,
                document_type: doc.document_type,
                document_name: doc.document_name.unwrap_or_default(),
                file_format: doc.file_format.unwrap_or_default(),
                file_size: doc.file_size.unwrap_or(0),
                version: doc.version,
                template_used: doc.template_used,
                status: doc.status,
                download_count: doc.download_count,
                download_url: Some(format!("/api/v1/documents/{}/download", doc.id)),
                generated_at: doc.generated_at.unwrap_or(doc.created_at),
                expires_at: doc.expires_at,
            })
            .collect();

        Ok(responses)
    }

    /// Download document
    pub async fn download_document(
        &self,
        user_id: Uuid,
        document_id: Uuid,
    ) -> Result<(String, Vec<u8>)> { // (filename, content)
        let doc: Option<GeneratedDocument> = sqlx::query_as(
            "SELECT d.* FROM generated_documents d
             JOIN businesses b ON d.business_id = b.id
             WHERE d.id = $1 AND b.user_id = $2 AND d.status = 'ready'"
        )
        .bind(document_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        let Some(doc) = doc else {
            return Err(AppError::NotFound("Document not found or not ready".to_string()));
        };

        // Increment download count
        sqlx::query(
            "UPDATE generated_documents SET download_count = download_count + 1 WHERE id = $1"
        )
        .bind(document_id)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        let file_data = doc.file_data.ok_or_else(|| {
            AppError::Internal("Document file data not found".to_string())
        })?;

        let filename = format!("{}.{}", 
            doc.document_name.unwrap_or_else(|| "document".to_string())
                .to_lowercase()
                .replace(" ", "-"),
            doc.file_format.unwrap_or_else(|| "pdf".to_string())
        );

        Ok((filename, file_data))
    }

    /// Get pitch deck templates
    pub async fn get_pitch_deck_templates(&self) -> Result<Vec<PitchDeckTemplate>> {
        Ok(get_pitch_deck_templates())
    }

    // =================================================================================
    // Background Generation Tasks
    // =================================================================================

    async fn generate_business_plan_background(
        db: PgPool,
        ai_service: Arc<AIService>,
        doc_id: Uuid,
        business: BusinessDetails,
        years: i32,
        include_financials: bool,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Generate each section using AI
        let executive_summary = generate_executive_summary(&ai_service, &business).await?;
        let problem_statement = generate_problem_statement(&ai_service, &business).await?;
        let solution = generate_solution_section(&ai_service, &business).await?;
        let market_analysis = generate_market_analysis(&ai_service, &business, years).await?;
        let competitive_analysis = generate_competitive_analysis(&ai_service, &business).await?;
        let business_model = generate_business_model_section(&ai_service, &business).await?;
        let management_team = generate_management_team(&ai_service, &business).await?;
        
        let financial_projections = if include_financials {
            Some(generate_financial_projections(&ai_service, &business, years).await?)
        } else {
            None
        };

        // Compile the full business plan
        let business_plan = BusinessPlanContent {
            executive_summary,
            company_overview: CompanyOverview {
                company_name: business.name.clone(),
                legal_structure: "Private Limited Company".to_string(),
                location: business.country.clone(),
                history: format!("{} was founded to address critical challenges in the {} industry.", business.name, business.industry),
                vision: business.vision.clone(),
                milestones: vec![
                    "Company founded".to_string(),
                    "MVP development started".to_string(),
                    "Initial market research completed".to_string(),
                ],
            },
            problem_statement,
            solution,
            market_analysis,
            competitive_analysis,
            business_model,
            marketing_sales: MarketingSales {
                go_to_market: "Digital-first approach targeting early adopters".to_string(),
                marketing_channels: vec![
                    "Social media marketing".to_string(),
                    "Content marketing".to_string(),
                    "Partnership channels".to_string(),
                ],
                sales_process: "Direct sales and self-service".to_string(),
                customer_lifecycle: "Acquire → Activate → Retain → Refer".to_string(),
                partnerships: vec!["Strategic industry partners".to_string()],
            },
            operations: OperationsSection {
                day_to_day: "Technology-enabled operations with lean team".to_string(),
                technology: "Cloud-based infrastructure with modern tech stack".to_string(),
                supply_chain: None,
                key_partners: vec!["Technology vendors".to_string(), "Distribution partners".to_string()],
                regulatory_compliance: format!("Compliant with {} regulations", business.country),
            },
            management_team,
            financial_projections: financial_projections.unwrap_or_default(),
            funding_request: None,
        };

        // Generate PDF content (simplified - in production use a PDF library)
        let pdf_content = format_business_plan_as_html(&business_plan, &business);
        let pdf_bytes = create_pdf_from_html(&pdf_content).await?;

        // Update document record
        let processing_time = start_time.elapsed().as_millis() as i32;
        
        sqlx::query(
            "UPDATE generated_documents SET
                file_data = $1,
                file_format = 'pdf',
                file_size = $2,
                status = 'ready',
                generation_params = $3,
                token_usage = $4,
                generated_at = NOW(),
                updated_at = NOW()
             WHERE id = $5"
        )
        .bind(&pdf_bytes)
        .bind(pdf_bytes.len() as i64)
        .bind(serde_json::json!({
            "years": years,
            "include_financials": include_financials,
            "content_summary": business_plan.executive_summary.mission_statement
        }))
        .bind(Some(processing_time / 10)) // Estimated tokens
        .bind(doc_id)
        .execute(&db)
        .await
        .map_err(AppError::Database)?;

        info!("Business plan generated for {} in {}ms", business.name, processing_time);

        Ok(())
    }

    async fn generate_pitch_deck_background(
        db: PgPool,
        ai_service: Arc<AIService>,
        doc_id: Uuid,
        business: BusinessDetails,
        template: String,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Generate slide content using AI
        let title_slide = PitchDeckSlide::Title {
            company_name: business.name.clone(),
            tagline: business.tagline.clone(),
            founder_names: "Founding Team".to_string(),
            contact_info: "contact@company.com".to_string(),
        };

        let problem_slide = generate_pitch_problem_slide(&ai_service, &business).await?;
        let solution_slide = generate_pitch_solution_slide(&ai_service, &business).await?;
        let market_slide = generate_pitch_market_slide(&ai_service, &business).await?;
        let business_model_slide = generate_pitch_business_model_slide(&ai_service, &business).await?;
        let team_slide = generate_pitch_team_slide(&ai_service, &business).await?;
        let traction_slide = generate_pitch_traction_slide(&ai_service, &business).await?;

        // Compile pitch deck
        let pitch_deck = PitchDeckContent {
            slides: vec![
                title_slide,
                problem_slide,
                solution_slide,
                market_slide,
                business_model_slide,
                traction_slide,
                team_slide,
            ],
            template: template.clone(),
        };

        // Generate PDF content
        let pdf_content = format_pitch_deck_as_html(&pitch_deck, &business, &template);
        let pdf_bytes = create_pdf_from_html(&pdf_content).await?;

        // Update document record
        let processing_time = start_time.elapsed().as_millis() as i32;

        sqlx::query(
            "UPDATE generated_documents SET
                file_data = $1,
                file_format = 'pdf',
                file_size = $2,
                status = 'ready',
                template_used = $3,
                generation_params = $4,
                generated_at = NOW(),
                updated_at = NOW()
             WHERE id = $5"
        )
        .bind(&pdf_bytes)
        .bind(pdf_bytes.len() as i64)
        .bind(&template)
        .bind(serde_json::json!({"slide_count": pitch_deck.slides.len()}))
        .bind(doc_id)
        .execute(&db)
        .await
        .map_err(AppError::Database)?;

        info!("Pitch deck generated for {} in {}ms", business.name, processing_time);

        Ok(())
    }

    // =================================================================================
    // Helper Methods
    // =================================================================================

    async fn get_business_details(&self, business_id: Uuid, user_id: Uuid) -> Result<BusinessDetails> {
        let row: Option<(String, String, Option<String>, String, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT name, industry, tagline, country, mission, vision 
             FROM businesses WHERE id = $1 AND user_id = $2"
        )
        .bind(business_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        row.map(|(name, industry, tagline, country, mission, vision)| BusinessDetails {
            id: business_id,
            name,
            industry,
            tagline: tagline.unwrap_or_default(),
            country,
            mission: mission.unwrap_or_default(),
            vision: vision.unwrap_or_default(),
        }).ok_or_else(|| AppError::NotFound("Business not found".to_string()))
    }
}

// =================================================================================
// Business Details Struct
// =================================================================================

#[derive(Clone)]
struct BusinessDetails {
    id: Uuid,
    name: String,
    industry: String,
    tagline: String,
    country: String,
    mission: String,
    vision: String,
}

// =================================================================================
// AI Content Generation Functions
// =================================================================================

async fn generate_executive_summary(
    ai_service: &Arc<AIService>,
    business: &BusinessDetails,
) -> Result<ExecutiveSummary> {
    let prompt = format!(
        "Generate a concise executive summary for {} ({} industry, {}).
        Mission: {}
        Vision: {}
        
        Provide JSON with: mission_statement, problem_summary, solution_summary, 
        market_opportunity, business_model_summary, team_overview",
        business.name, business.industry, business.country,
        business.mission, business.vision
    );

    let response = ai_service.generate_text(
        "You are a professional business plan writer. Generate concise, investor-ready content.",
        &prompt,
        1000
    ).await.map_err(|e| AppError::AiGeneration(e.to_string()))?;

    // Parse JSON response
    serde_json::from_str(&response)
        .map_err(|_| AppError::AiGeneration("Failed to parse executive summary".to_string()))
}

async fn generate_problem_statement(
    ai_service: &Arc<AIService>,
    business: &BusinessDetails,
) -> Result<ProblemStatement> {
    let prompt = format!(
        "Generate a problem statement for {} in the {} industry targeting {}.
        Provide JSON with: problem_description, who_experiences_it, current_solutions, gaps_in_current_solutions, cost_of_problem",
        business.name, business.industry, business.country
    );

    let response = ai_service.generate_text(
        "You are a business analyst. Clearly articulate the problem being solved.",
        &prompt,
        800
    ).await.map_err(|e| AppError::AiGeneration(e.to_string()))?;

    serde_json::from_str(&response)
        .map_err(|_| AppError::AiGeneration("Failed to parse problem statement".to_string()))
}

async fn generate_solution_section(
    ai_service: &Arc<AIService>,
    business: &BusinessDetails,
) -> Result<SolutionSection> {
    let prompt = format!(
        "Generate solution description for {} ({} industry).
        Tagline: {}
        
        Provide JSON with: product_description, how_it_solves, unique_value_proposition,
        key_features (array of {{name, description}}), benefits (array of strings)",
        business.name, business.industry, business.tagline
    );

    let response = ai_service.generate_text(
        "You are a product strategist. Describe the solution compellingly.",
        &prompt,
        1000
    ).await.map_err(|e| AppError::AiGeneration(e.to_string()))?;

    serde_json::from_str(&response)
        .map_err(|_| AppError::AiGeneration("Failed to parse solution".to_string()))
}

async fn generate_market_analysis(
    ai_service: &Arc<AIService>,
    business: &BusinessDetails,
    years: i32,
) -> Result<MarketAnalysis> {
    let prompt = format!(
        "Generate market analysis for {} in {} over {} years.
        Provide JSON with: tam {{value, explanation}}, sam {{value, explanation}}, 
        som {{value, explanation}}, market_trends (array), 
        target_segments (array of {{name, description, size}}), growth_projections",
        business.industry, business.country, years
    );

    let response = ai_service.generate_text(
        "You are a market research analyst. Provide realistic market sizing.",
        &prompt,
        1200
    ).await.map_err(|e| AppError::AiGeneration(e.to_string()))?;

    serde_json::from_str(&response)
        .map_err(|_| AppError::AiGeneration("Failed to parse market analysis".to_string()))
}

async fn generate_competitive_analysis(
    ai_service: &Arc<AIService>,
    business: &BusinessDetails,
) -> Result<CompetitiveAnalysis> {
    let prompt = format!(
        "Generate competitive analysis for {} in {} industry.
        Provide JSON with: direct_competitors (array of {{name, description, strengths, weaknesses}}),
        indirect_competitors (array), competitive_advantage, barriers_to_entry (array)",
        business.name, business.industry
    );

    let response = ai_service.generate_text(
        "You are a competitive strategist. Be objective and thorough.",
        &prompt,
        1000
    ).await.map_err(|e| AppError::AiGeneration(e.to_string()))?;

    serde_json::from_str(&response)
        .map_err(|_| AppError::AiGeneration("Failed to parse competitive analysis".to_string()))
}

async fn generate_business_model_section(
    ai_service: &Arc<AIService>,
    business: &BusinessDetails,
) -> Result<BusinessModelSection> {
    let prompt = format!(
        "Generate business model for {} in {} industry.
        Provide JSON with: revenue_streams (array of {{name, description, percentage}}),
        pricing_strategy, unit_economics, sales_channels (array), customer_acquisition",
        business.name, business.industry
    );

    let response = ai_service.generate_text(
        "You are a business model expert. Be specific about revenue generation.",
        &prompt,
        800
    ).await.map_err(|e| AppError::AiGeneration(e.to_string()))?;

    serde_json::from_str(&response)
        .map_err(|_| AppError::AiGeneration("Failed to parse business model".to_string()))
}

async fn generate_management_team(
    _ai_service: &Arc<AIService>,
    _business: &BusinessDetails,
) -> Result<ManagementTeam> {
    // For now, return placeholder team
    Ok(ManagementTeam {
        founders: vec![
            TeamMember {
                name: "Founder".to_string(),
                role: "CEO & Co-founder".to_string(),
                bio: "Experienced entrepreneur with industry expertise.".to_string(),
                photo_url: None,
            }
        ],
        key_members: vec![],
        advisors: vec![],
        hiring_plan: "Build core team over 12 months".to_string(),
        org_structure: "Flat hierarchy with clear responsibilities".to_string(),
    })
}

async fn generate_financial_projections(
    ai_service: &Arc<AIService>,
    business: &BusinessDetails,
    years: i32,
) -> Result<FinancialProjections> {
    let prompt = format!(
        "Generate {}-year financial projections for {} in {} industry in {}.
        Provide JSON with: years, revenue_projections (array of {{year, revenue, expenses, profit, growth_rate}}),
        expense_breakdown (array of {{category, percentage, description}}), profit_loss_summary,
        cash_flow_summary, key_assumptions (array), break_even_analysis",
        years, business.name, business.industry, business.country
    );

    let response = ai_service.generate_text(
        "You are a financial analyst. Create realistic projections with conservative estimates.",
        &prompt,
        1200
    ).await.map_err(|e| AppError::AiGeneration(e.to_string()))?;

    serde_json::from_str(&response)
        .map_err(|_| AppError::AiGeneration("Failed to parse financial projections".to_string()))
}

// Pitch deck slide generators
async fn generate_pitch_problem_slide(
    ai_service: &Arc<AIService>,
    business: &BusinessDetails,
) -> Result<PitchDeckSlide> {
    let prompt = format!(
        "Generate pitch deck 'Problem' slide content for {} ({}).
        Provide JSON with: problem_statement, visual_description, market_size",
        business.name, business.industry
    );

    let response = ai_service.generate_text(
        "You are creating a pitch deck. Be concise and impactful.",
        &prompt,
        400
    ).await.map_err(|e| AppError::AiGeneration(e.to_string()))?;

    let content: serde_json::Value = serde_json::from_str(&response)
        .map_err(|_| AppError::AiGeneration("Failed to parse problem slide".to_string()))?;

    Ok(PitchDeckSlide::Problem {
        title: "The Problem".to_string(),
        content,
    })
}

async fn generate_pitch_solution_slide(
    ai_service: &Arc<AIService>,
    business: &BusinessDetails,
) -> Result<PitchDeckSlide> {
    let prompt = format!(
        "Generate pitch deck 'Solution' slide content for {}. Tagline: {}
        Provide JSON with: solution_description, key_benefits (array), visual_description",
        business.name, business.tagline
    );

    let response = ai_service.generate_text(
        "You are creating a pitch deck. Be concise and impactful.",
        &prompt,
        400
    ).await.map_err(|e| AppError::AiGeneration(e.to_string()))?;

    let content: serde_json::Value = serde_json::from_str(&response)
        .map_err(|_| AppError::AiGeneration("Failed to parse solution slide".to_string()))?;

    Ok(PitchDeckSlide::Solution {
        title: "Our Solution".to_string(),
        content,
    })
}

async fn generate_pitch_market_slide(
    ai_service: &Arc<AIService>,
    business: &BusinessDetails,
) -> Result<PitchDeckSlide> {
    let prompt = format!(
        "Generate pitch deck 'Market Opportunity' slide for {} in {}.
        Provide JSON with: tam, sam, som, growth_rate, target_customer",
        business.industry, business.country
    );

    let response = ai_service.generate_text(
        "You are creating a pitch deck. Show the opportunity clearly.",
        &prompt,
        400
    ).await.map_err(|e| AppError::AiGeneration(e.to_string()))?;

    let content: serde_json::Value = serde_json::from_str(&response)
        .map_err(|_| AppError::AiGeneration("Failed to parse market slide".to_string()))?;

    Ok(PitchDeckSlide::Market {
        title: "Market Opportunity".to_string(),
        content,
    })
}

async fn generate_pitch_business_model_slide(
    ai_service: &Arc<AIService>,
    business: &BusinessDetails,
) -> Result<PitchDeckSlide> {
    let prompt = format!(
        "Generate pitch deck 'Business Model' slide for {}.
        Provide JSON with: how_we_make_money, pricing_tiers (array of {{name, price, features}}), unit_economics",
        business.name
    );

    let response = ai_service.generate_text(
        "You are creating a pitch deck. Show how you make money.",
        &prompt,
        400
    ).await.map_err(|e| AppError::AiGeneration(e.to_string()))?;

    let content: serde_json::Value = serde_json::from_str(&response)
        .map_err(|_| AppError::AiGeneration("Failed to parse business model slide".to_string()))?;

    Ok(PitchDeckSlide::BusinessModel {
        title: "Business Model".to_string(),
        content,
    })
}

async fn generate_pitch_team_slide(
    _ai_service: &Arc<AIService>,
    _business: &BusinessDetails,
) -> Result<PitchDeckSlide> {
    Ok(PitchDeckSlide::Team {
        title: "Our Team".to_string(),
        members: vec![
            serde_json::json!({
                "name": "Founding Team",
                "role": "Industry Experts",
                "bio": "Experienced professionals with deep industry knowledge"
            })
        ],
    })
}

async fn generate_pitch_traction_slide(
    ai_service: &Arc<AIService>,
    business: &BusinessDetails,
) -> Result<PitchDeckSlide> {
    let prompt = format!(
        "Generate pitch deck 'Traction' slide for {}.
        Provide JSON with: key_achievements (array), metrics (array of {{name, value, timeframe}}), partnerships (array), press (array)",
        business.name
    );

    let response = ai_service.generate_text(
        "You are creating a pitch deck. Show progress and validation.",
        &prompt,
        400
    ).await.map_err(|e| AppError::AiGeneration(e.to_string()))?;

    let content: serde_json::Value = serde_json::from_str(&response)
        .map_err(|_| AppError::AiGeneration("Failed to parse traction slide".to_string()))?;

    Ok(PitchDeckSlide::Traction {
        title: "Traction".to_string(),
        content,
    })
}

// =================================================================================
// PDF Generation Helpers (Simplified)
// =================================================================================

fn format_business_plan_as_html(plan: &BusinessPlanContent, business: &BusinessDetails) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{} - Business Plan</title>
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; margin: 40px; }}
        h1 {{ color: #2563EB; border-bottom: 2px solid #2563EB; }}
        h2 {{ color: #1E40AF; margin-top: 30px; }}
        .section {{ margin: 20px 0; }}
        .highlight {{ background: #EFF6FF; padding: 15px; border-radius: 5px; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <p><strong>Industry:</strong> {} | <strong>Location:</strong> {}</p>
    
    <div class="section">
        <h2>Executive Summary</h2>
        <div class="highlight">
            <p><strong>Mission:</strong> {}</p>
            <p><strong>Market Opportunity:</strong> {}</p>
        </div>
    </div>
    
    <div class="section">
        <h2>Problem Statement</h2>
        <p>{}</p>
    </div>
    
    <div class="section">
        <h2>Solution</h2>
        <p><strong>Value Proposition:</strong> {}</p>
    </div>
    
    <div class="section">
        <h2>Market Analysis</h2>
        <p><strong>TAM:</strong> {}</p>
        <p><strong>SAM:</strong> {}</p>
        <p><strong>SOM:</strong> {}</p>
    </div>
</body>
</html>"#,
        business.name,
        business.name,
        business.industry,
        business.country,
        plan.executive_summary.mission_statement,
        plan.executive_summary.market_opportunity,
        plan.problem_statement.problem_description,
        plan.solution.unique_value_proposition,
        plan.market_analysis.tam.value,
        plan.market_analysis.sam.value,
        plan.market_analysis.som.value,
    )
}

fn format_pitch_deck_as_html(deck: &PitchDeckContent, business: &BusinessDetails, template: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{} - Pitch Deck</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 0; }}
        .slide {{ width: 100%; height: 100vh; display: flex; flex-direction: column; justify-content: center; align-items: center; padding: 60px; box-sizing: border-box; page-break-after: always; }}
        .slide-title {{ background: #2563EB; color: white; }}
        h1 {{ font-size: 48px; margin: 0; }}
        h2 {{ font-size: 36px; color: #2563EB; }}
        .tagline {{ font-size: 24px; color: #4B5563; margin-top: 20px; }}
    </style>
</head>
<body>
    <div class="slide slide-title">
        <h1>{}</h1>
        <p class="tagline">{}</p>
    </div>
    {}
</body>
</html>"#,
        business.name,
        business.name,
        business.tagline,
        deck.slides.iter().map(|slide| format_slide_html(slide)).collect::<String>()
    )
}

fn format_slide_html(slide: &PitchDeckSlide) -> String {
    match slide {
        PitchDeckSlide::Title { .. } => "".to_string(), // Already handled
        PitchDeckSlide::Problem { title, content } => format!(
            r#"<div class="slide"><h2>{}</h2><p>{}</p></div>"#,
            title,
            content.get("problem_statement").and_then(|v| v.as_str()).unwrap_or("Problem description here")
        ),
        PitchDeckSlide::Solution { title, content } => format!(
            r#"<div class="slide"><h2>{}</h2><p>{}</p></div>"#,
            title,
            content.get("solution_description").and_then(|v| v.as_str()).unwrap_or("Solution description here")
        ),
        _ => r#"<div class="slide"><h2>Coming Soon</h2></div>"#.to_string(),
    }
}

async fn create_pdf_from_html(html: &str) -> Result<Vec<u8>> {
    // In production, use a proper PDF generation library like headless_chrome or puppeteer
    // For now, return HTML as bytes with a PDF header marker
    let mut result = b"%PDF-1.4\n".to_vec();
    result.extend_from_slice(html.as_bytes());
    Ok(result)
}


