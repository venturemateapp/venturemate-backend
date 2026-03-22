//! Health Score Service
//!
//! Calculates and manages the Startup Health Score™ across
//! multiple dimensions: compliance, revenue, market fit, team, operations.

use crate::utils::{AppError, Result};
use crate::models::health_score::{
    ComplianceScoreBreakdown, ContributingFactors, HealthScore, HealthScoreHistory, HealthScoreHistoryPoint,
    MarketFitScoreBreakdown, OperationsScoreBreakdown, RevenueScoreBreakdown,
    TeamScoreBreakdown, WebsiteAnalysisResult,
};
use crate::services::ai_service::AIService;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub struct HealthScoreService {
    db: PgPool,
    ai_service: Arc<AIService>,
}

impl HealthScoreService {
    pub fn new(db: PgPool, ai_service: Arc<AIService>) -> Self {
        Self { db, ai_service }
    }

    /// Get health score for a business
    pub async fn get_health_score(&self, business_id: Uuid) -> Result<Option<HealthScore>> {
        let score: Option<HealthScore> = sqlx::query_as(
            "SELECT * FROM health_scores WHERE business_id = $1"
        )
        .bind(business_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(score)
    }

    /// Calculate or refresh health score
    pub async fn calculate_health_score(&self, business_id: Uuid) -> Result<HealthScore> {
        info!("Calculating health score for business: {}", business_id);

        // Calculate each component score
        let compliance_score = self.calculate_compliance_score(business_id).await?;
        let revenue_score = self.calculate_revenue_score(business_id).await?;
        let market_fit_score = self.calculate_market_fit_score(business_id).await?;
        let team_score = self.calculate_team_score(business_id).await?;
        let operations_score = self.calculate_operations_score(business_id).await?;

        // Calculate funding readiness (derived from other scores)
        let funding_readiness_score = self.calculate_funding_readiness(
            compliance_score.score,
            revenue_score.score,
            market_fit_score.score,
            team_score.score,
            operations_score.score,
        );

        // Calculate overall score with weights
        let overall_score = (
            compliance_score.score as f32 * 0.25 +
            revenue_score.score as f32 * 0.25 +
            market_fit_score.score as f32 * 0.20 +
            team_score.score as f32 * 0.15 +
            operations_score.score as f32 * 0.15
        ) as i32;

        // Build contributing factors
        let contributing_factors = self.build_contributing_factors(
            &compliance_score,
            &revenue_score,
            &market_fit_score,
            &team_score,
            &operations_score,
        );

        // Build score breakdown
        let score_breakdown = serde_json::json!({
            "compliance": compliance_score.breakdown,
            "revenue": revenue_score.breakdown,
            "market_fit": market_fit_score.breakdown,
            "team": team_score.breakdown,
            "operations": operations_score.breakdown,
        });

        // Upsert health score
        let health_score = sqlx::query_as(
            r#"
            INSERT INTO health_scores (
                business_id, overall_score, compliance_score, revenue_score, market_fit_score,
                team_score, operations_score, funding_readiness_score, score_breakdown,
                contributing_factors, calculated_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW(), NOW())
            ON CONFLICT (business_id) DO UPDATE SET
                overall_score = EXCLUDED.overall_score,
                compliance_score = EXCLUDED.compliance_score,
                revenue_score = EXCLUDED.revenue_score,
                market_fit_score = EXCLUDED.market_fit_score,
                team_score = EXCLUDED.team_score,
                operations_score = EXCLUDED.operations_score,
                funding_readiness_score = EXCLUDED.funding_readiness_score,
                score_breakdown = EXCLUDED.score_breakdown,
                contributing_factors = EXCLUDED.contributing_factors,
                calculated_at = NOW(),
                updated_at = NOW()
            RETURNING *
            "#
        )
        .bind(business_id)
        .bind(overall_score)
        .bind(compliance_score.score)
        .bind(revenue_score.score)
        .bind(market_fit_score.score)
        .bind(team_score.score)
        .bind(operations_score.score)
        .bind(funding_readiness_score)
        .bind(score_breakdown)
        .bind(serde_json::to_value(&contributing_factors).unwrap_or_default())
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        // Log to history
        sqlx::query(
            "INSERT INTO health_score_history (business_id, overall_score, compliance_score, revenue_score, market_fit_score, team_score, operations_score, funding_readiness_score, calculated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())"
        )
        .bind(business_id)
        .bind(overall_score)
        .bind(compliance_score.score)
        .bind(revenue_score.score)
        .bind(market_fit_score.score)
        .bind(team_score.score)
        .bind(operations_score.score)
        .bind(funding_readiness_score)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        info!("Health score calculated for business {}: overall={}", business_id, overall_score);

        Ok(health_score)
    }

    /// Get health score history
    pub async fn get_score_history(&self, business_id: Uuid, days: i32) -> Result<Vec<HealthScoreHistoryPoint>> {
        let history: Vec<HealthScoreHistory> = sqlx::query_as(
            "SELECT * FROM health_score_history 
             WHERE business_id = $1 AND calculated_at > NOW() - INTERVAL '$2 days'
             ORDER BY calculated_at ASC"
        )
        .bind(business_id)
        .bind(days)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(history.into_iter().map(|h| HealthScoreHistoryPoint {
            date: h.calculated_at,
            overall_score: h.overall_score,
            compliance_score: h.compliance_score,
            revenue_score: h.revenue_score,
            market_fit_score: h.market_fit_score,
            team_score: h.team_score,
            operations_score: h.operations_score,
        }).collect())
    }

    /// Calculate compliance score
    async fn calculate_compliance_score(&self, business_id: Uuid) -> Result<ComponentScore> {
        let mut score = 0;
        let mut breakdown = ComplianceScoreBreakdown {
            business_registration: 0,
            tax_id: 0,
            industry_licenses: 0,
            document_vault: 0,
            legal_structure: 0,
        };

        // Check business registration status
        let registration_status: Option<String> = sqlx::query_scalar(
            "SELECT status FROM business_registrations WHERE business_id = $1 ORDER BY created_at DESC LIMIT 1"
        )
        .bind(business_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        breakdown.business_registration = match registration_status.as_deref() {
            Some("approved") => 30,
            Some("submitted") | Some("pending") => 15,
            _ => 0,
        };
        score += breakdown.business_registration;

        // Check for tax ID
        let has_tax_id: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM tax_ids WHERE business_id = $1)"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        breakdown.tax_id = if has_tax_id { 20 } else { 0 };
        score += breakdown.tax_id;

        // Check document vault
        let doc_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM documents WHERE business_id = $1"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        breakdown.document_vault = if doc_count >= 5 { 15 } else if doc_count >= 3 { 8 } else { 0 };
        score += breakdown.document_vault;

        // Legal structure (check if business has valid structure)
        let legal_structure: Option<String> = sqlx::query_scalar(
            "SELECT legal_structure FROM businesses WHERE id = $1"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        breakdown.legal_structure = if legal_structure.is_some() && legal_structure.as_deref() != Some("none") {
            15
        } else {
            0
        };
        score += breakdown.legal_structure;

        // Industry licenses (simplified - check if any licenses exist)
        let license_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM regulatory_compliance_items WHERE business_id = $1 AND status = 'completed'"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        breakdown.industry_licenses = if license_count > 0 { 20 } else { 0 };
        score += breakdown.industry_licenses;

        Ok(ComponentScore {
            score,
            weight: 0.25,
            breakdown: Some(serde_json::to_value(&breakdown).unwrap_or_default()),
            max_score: Some(100),
            grade: None,
            status: None,
        })
    }

    /// Calculate revenue score
    async fn calculate_revenue_score(&self, business_id: Uuid) -> Result<ComponentScore> {
        let mut score = 0;
        let mut breakdown = RevenueScoreBreakdown {
            bank_account_connected: 0,
            payment_gateway: 0,
            invoices_created: 0,
            revenue_generated: 0,
            financial_projections: 0,
        };

        // Check bank connection
        let bank_connected: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM bank_connections WHERE business_id = $1 AND status = 'connected')"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        breakdown.bank_account_connected = if bank_connected { 25 } else { 0 };
        score += breakdown.bank_account_connected;

        // Check payment gateway
        let payment_gateway: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM payment_gateways WHERE business_id = $1 AND status = 'active')"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        breakdown.payment_gateway = if payment_gateway { 25 } else { 0 };
        score += breakdown.payment_gateway;

        // Check invoices created
        let invoice_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM invoices WHERE business_id = $1"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        breakdown.invoices_created = if invoice_count > 10 { 20 } else if invoice_count >= 5 { 15 } else if invoice_count >= 1 { 10 } else { 0 };
        score += breakdown.invoices_created;

        // Check financial projections (generated documents)
        let has_projections: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM generated_documents WHERE business_id = $1 AND document_type = 'financial_model' AND status = 'ready')"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        breakdown.financial_projections = if has_projections { 10 } else { 0 };
        score += breakdown.financial_projections;

        // Revenue generated (placeholder - would need actual revenue tracking)
        breakdown.revenue_generated = 20; // Assume some revenue for now
        score += breakdown.revenue_generated;

        Ok(ComponentScore {
            score,
            weight: 0.25,
            breakdown: Some(serde_json::to_value(&breakdown).unwrap_or_default()),
            max_score: Some(100),
            grade: None,
            status: None,
        })
    }

    /// Calculate market fit score
    async fn calculate_market_fit_score(&self, business_id: Uuid) -> Result<ComponentScore> {
        let mut score = 0;
        let mut breakdown = MarketFitScoreBreakdown {
            website_quality: 0,
            brand_identity: 0,
            marketing_copy: 0,
            social_media_presence: 0,
        };

        // Check website
        let website: Option<(String,)> = sqlx::query_as(
            "SELECT status FROM websites WHERE business_id = $1"
        )
        .bind(business_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        breakdown.website_quality = if website.map(|w| w.0) == Some("published".to_string()) { 35 } else { 0 };
        score += breakdown.website_quality;

        // Check brand assets
        let brand_assets: Option<(Option<serde_json::Value>,)> = sqlx::query_as(
            "SELECT logo_variants FROM brand_assets WHERE business_id = $1 AND status = 'ready'"
        )
        .bind(business_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        breakdown.brand_identity = if brand_assets.is_some() { 25 } else { 0 };
        score += breakdown.brand_identity;

        // Check for marketing copy (AI generated content)
        let has_marketing_copy: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM ai_content WHERE business_id = $1)"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        breakdown.marketing_copy = if has_marketing_copy { 20 } else { 0 };
        score += breakdown.marketing_copy;

        // Social media presence (placeholder)
        breakdown.social_media_presence = 20;
        score += breakdown.social_media_presence;

        Ok(ComponentScore {
            score,
            weight: 0.20,
            breakdown: Some(serde_json::to_value(&breakdown).unwrap_or_default()),
            max_score: Some(100),
            grade: None,
            status: None,
        })
    }

    /// Calculate team score
    async fn calculate_team_score(&self, business_id: Uuid) -> Result<ComponentScore> {
        let mut score = 0;
        let mut breakdown = TeamScoreBreakdown {
            founder_completeness: 0,
            key_roles_filled: 0,
            advisory_board: 0,
            team_documents: 0,
        };

        // Get business owner/founder info
        let user_id: Uuid = sqlx::query_scalar(
            "SELECT user_id FROM businesses WHERE id = $1"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        // Check founder profile completeness
        let profile: Option<(Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT first_name, last_name FROM user_profiles WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        breakdown.founder_completeness = if profile.is_some() { 40 } else { 20 };
        score += breakdown.founder_completeness;

        // Key roles filled (placeholder - would check team_members table)
        breakdown.key_roles_filled = 20;
        score += breakdown.key_roles_filled;

        // Advisory board (placeholder)
        breakdown.advisory_board = 10;
        score += breakdown.advisory_board;

        // Team documents (placeholder)
        breakdown.team_documents = 5;
        score += breakdown.team_documents;

        Ok(ComponentScore {
            score,
            weight: 0.15,
            breakdown: Some(serde_json::to_value(&breakdown).unwrap_or_default()),
            max_score: Some(100),
            grade: None,
            status: None,
        })
    }

    /// Calculate operations score
    async fn calculate_operations_score(&self, business_id: Uuid) -> Result<ComponentScore> {
        let mut score = 0;
        let mut breakdown = OperationsScoreBreakdown {
            crm_setup: 0,
            document_management: 0,
            tools_integration: 0,
            processes_defined: 0,
        };

        // CRM setup (check contacts)
        let contact_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM crm_contacts WHERE business_id = $1"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        breakdown.crm_setup = if contact_count > 10 { 25 } else if contact_count > 0 { 15 } else { 0 };
        score += breakdown.crm_setup;

        // Document management
        let doc_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM documents WHERE business_id = $1"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        breakdown.document_management = if doc_count >= 10 { 25 } else if doc_count >= 5 { 15 } else { 0 };
        score += breakdown.document_management;

        // Tools integration (placeholder)
        breakdown.tools_integration = 20;
        score += breakdown.tools_integration;

        // Processes defined (placeholder)
        breakdown.processes_defined = 10;
        score += breakdown.processes_defined;

        Ok(ComponentScore {
            score,
            weight: 0.15,
            breakdown: Some(serde_json::to_value(&breakdown).unwrap_or_default()),
            max_score: Some(100),
            grade: None,
            status: None,
        })
    }

    /// Calculate funding readiness score
    fn calculate_funding_readiness(
        &self,
        compliance: i32,
        revenue: i32,
        market_fit: i32,
        team: i32,
        operations: i32,
    ) -> i32 {
        // Funding readiness is based on having solid foundations across all areas
        let avg = (compliance + revenue + market_fit + team + operations) as f32 / 5.0;
        
        // Boost if all components are above threshold
        if compliance >= 60 && revenue >= 40 && market_fit >= 50 && team >= 40 && operations >= 50 {
            (avg * 1.2) as i32
        } else {
            avg as i32
        }
    }

    /// Build contributing factors list
    fn build_contributing_factors(
        &self,
        compliance: &ComponentScore,
        revenue: &ComponentScore,
        market_fit: &ComponentScore,
        team: &ComponentScore,
        _operations: &ComponentScore,
    ) -> ContributingFactors {
        let mut positive = Vec::new();
        let mut negative = Vec::new();

        if compliance.score >= 60 {
            positive.push("Good compliance foundation".to_string());
        } else {
            negative.push("Compliance requirements incomplete".to_string());
        }

        if revenue.score >= 50 {
            positive.push("Revenue systems in place".to_string());
        } else {
            negative.push("Revenue generation needs attention".to_string());
        }

        if market_fit.score >= 50 {
            positive.push("Strong market presence".to_string());
        } else {
            negative.push("Market fit could be improved".to_string());
        }

        if team.score >= 50 {
            positive.push("Team structure established".to_string());
        } else {
            negative.push("Team needs development".to_string());
        }

        ContributingFactors { positive, negative }
    }

    /// Analyze website using AI
    pub async fn analyze_website(&self, business_id: Uuid, website_url: String) -> Result<WebsiteAnalysisResult> {
        let prompt = format!(
            r#"You are evaluating the website for a startup. Score from 0-100 based on:

1. Clarity of Value Proposition (0-35)
   - Is it immediately clear what the business does?
   - Is the target customer identified?
   - Is the problem/solution clearly stated?

2. Professional Design (0-25)
   - Is the design modern and professional?
   - Is branding consistent?
   - Is it mobile-responsive?

3. Messaging & Copy (0-20)
   - Is the copy compelling?
   - Does it speak to the target audience?
   - Are there clear calls-to-action?

4. Trust Signals (0-20)
   - Contact information present?
   - About page with team?
   - Testimonials or social proof?

Website URL: {}

Return JSON with: clarity_score, design_score, messaging_score, trust_score, overall_score, recommendations (array), strengths (array), weaknesses (array)"#,
            website_url
        );

        let response = self.ai_service.generate_text(
            "You are a professional website reviewer. Be objective and thorough.",
            &prompt,
            1000,
            Some(0.7)
        ).await.map_err(|e| AppError::AiGeneration(e.to_string()))?;

        // Parse the JSON response
        let result: WebsiteAnalysisResult = serde_json::from_str(&response)
            .map_err(|_| AppError::AiGeneration("Failed to parse website analysis".to_string()))?;

        // Save the analysis
        sqlx::query(
            "INSERT INTO market_fit_analysis (business_id, analysis_type, content_url, ai_analysis, score_contribution, analyzed_at)
             VALUES ($1, 'website_review', $2, $3, $4, NOW())"
        )
        .bind(business_id)
        .bind(&website_url)
        .bind(serde_json::to_value(&result).unwrap_or_default())
        .bind(result.overall_score)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(result)
    }
}

use crate::models::health_score::ComponentScore;
