//! Smart Recommendations Engine Service
//!
//! Generates personalized, AI-powered recommendations
//! to guide founders toward startup success.

use crate::utils::{AppError, Result};
use crate::models::recommendations::{
    ActOnRecommendationRequest, ActOnRecommendationResponse, DismissRecommendationRequest,
    DismissRecommendationResponse, ListRecommendationsRequest, ListRecommendationsResponse,
    Recommendation, RecommendationAction, RecommendationContent, RecommendationContentRequest,
    RecommendationPriority, RecommendationResponse, RecommendationStatus, RecommendationTemplate,
    RecommendationType, RefreshRecommendationsResponse, TriggerCondition, get_recommendation_templates,
};
use crate::services::ai_service::AIService;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct RecommendationsService {
    db: PgPool,
    ai_service: Arc<AIService>,
}

impl RecommendationsService {
    pub fn new(db: PgPool, ai_service: Arc<AIService>) -> Self {
        Self { db, ai_service }
    }

    /// List recommendations for a business
    pub async fn list_recommendations(
        &self,
        business_id: Uuid,
        req: ListRecommendationsRequest,
    ) -> Result<ListRecommendationsResponse> {
        let status_filter = req.status.as_deref().unwrap_or("pending");

        let recommendations: Vec<Recommendation> = sqlx::query_as(
            "SELECT * FROM recommendations 
             WHERE business_id = $1 AND status = $2
             ORDER BY priority_score DESC, created_at DESC
             LIMIT 10"
        )
        .bind(business_id)
        .bind(status_filter)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::Database)?;

        let dismissed_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM recommendations WHERE business_id = $1 AND status = 'dismissed'"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        let total_pending: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM recommendations WHERE business_id = $1 AND status = 'pending'"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        let responses = recommendations.into_iter().map(|r| RecommendationResponse {
            id: r.id,
            recommendation_type: r.recommendation_type,
            title: r.title,
            description: r.description,
            impact_description: r.impact_description,
            cta_text: r.cta_text,
            cta_link: r.cta_link,
            priority: r.priority.clone(),
            priority_label: format!("{} {:?}", RecommendationPriority::from_str(&r.priority).emoji(), r.priority),
            status: r.status,
            created_at: r.created_at,
        }).collect();

        Ok(ListRecommendationsResponse {
            recommendations: responses,
            dismissed_count,
            total_pending,
        })
    }

    /// Refresh/generate new recommendations
    pub async fn refresh_recommendations(&self, business_id: Uuid) -> Result<RefreshRecommendationsResponse> {
        info!("Refreshing recommendations for business: {}", business_id);

        // Get business details
        let business: Option<(String, String, Option<String>)> = sqlx::query_as(
            "SELECT name, industry, business_stage FROM businesses WHERE id = $1"
        )
        .bind(business_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        let Some((name, industry, stage)) = business else {
            return Err(AppError::NotFound("Business not found".to_string()));
        };

        // Get health score for context
        let health_score: Option<(i32, i32, i32, i32, i32, i32)> = sqlx::query_as(
            "SELECT overall_score, compliance_score, revenue_score, market_fit_score, team_score, operations_score 
             FROM health_scores WHERE business_id = $1"
        )
        .bind(business_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        // Check for existing pending recommendations
        let existing_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM recommendations WHERE business_id = $1 AND status = 'pending'"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        if existing_count >= 5 {
            return Ok(RefreshRecommendationsResponse {
                new_recommendations_count: 0,
                message: "Maximum pending recommendations reached. Act on existing ones first.".to_string(),
            });
        }

        // Generate recommendations based on triggers
        let templates = get_recommendation_templates();
        let mut new_count = 0;

        for template in templates {
            // Check if recommendation already exists for this trigger
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM recommendations WHERE business_id = $1 AND trigger_source = $2 AND status = 'pending')"
            )
            .bind(business_id)
            .bind(&template.title_template)
            .fetch_one(&self.db)
            .await
            .map_err(AppError::Database)?;

            if exists {
                continue;
            }

            // Check trigger condition
            if self.check_trigger(&template.trigger_condition, business_id, health_score).await? {
                // Generate AI content
                let content_req = RecommendationContentRequest {
                    startup_name: name.clone(),
                    industry: industry.clone(),
                    business_stage: stage.clone().unwrap_or_else(|| "early".to_string()),
                    trigger_type: template.recommendation_type.as_str().to_string(),
                    trigger_context: serde_json::json!({}),
                };

                let content = self.generate_recommendation_content(content_req).await?;

                // Create recommendation
                let rec_id = Uuid::new_v4();
                sqlx::query(
                    "INSERT INTO recommendations (id, business_id, recommendation_type, trigger_source, title, description, impact_description, cta_text, cta_link, priority, status, priority_score, has_financial_impact, created_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'pending', $11, $12, NOW())"
                )
                .bind(rec_id)
                .bind(business_id)
                .bind(template.recommendation_type.as_str())
                .bind(&template.title_template)
                .bind(content.title)
                .bind(content.description)
                .bind(Some(content.impact))
                .bind(Some(template.cta_text))
                .bind(Some(template.cta_link))
                .bind(template.priority.as_str())
                .bind(template.priority.score())
                .bind(template.recommendation_type == RecommendationType::Revenue)
                .execute(&self.db)
                .await
                .map_err(AppError::Database)?;

                new_count += 1;
            }
        }

        info!("Generated {} new recommendations for business {}", new_count, business_id);

        Ok(RefreshRecommendationsResponse {
            new_recommendations_count: new_count,
            message: format!("Generated {} new recommendations", new_count),
        })
    }

    /// Dismiss a recommendation
    pub async fn dismiss_recommendation(
        &self,
        recommendation_id: Uuid,
        _user_id: Uuid,
        _req: DismissRecommendationRequest,
    ) -> Result<DismissRecommendationResponse> {
        sqlx::query(
            "UPDATE recommendations SET status = 'dismissed', dismissed_at = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind(recommendation_id)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        // Log the action
        sqlx::query(
            "INSERT INTO recommendation_actions (recommendation_id, user_id, action, created_at) VALUES ($1, $2, 'dismissed', NOW())"
        )
        .bind(recommendation_id)
        .bind(_user_id)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(DismissRecommendationResponse {
            success: true,
            message: "Recommendation dismissed".to_string(),
        })
    }

    /// Mark recommendation as acted upon
    pub async fn act_on_recommendation(
        &self,
        recommendation_id: Uuid,
        user_id: Uuid,
        _req: ActOnRecommendationRequest,
    ) -> Result<ActOnRecommendationResponse> {
        sqlx::query(
            "UPDATE recommendations SET status = 'acted', acted_at = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind(recommendation_id)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        // Log the action
        sqlx::query(
            "INSERT INTO recommendation_actions (recommendation_id, user_id, action, created_at) VALUES ($1, $2, 'acted', NOW())"
        )
        .bind(recommendation_id)
        .bind(user_id)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(ActOnRecommendationResponse {
            success: true,
            message: "Action recorded".to_string(),
        })
    }

    /// Check trigger condition
    async fn check_trigger(
        &self,
        condition: &TriggerCondition,
        business_id: Uuid,
        health_score: Option<(i32, i32, i32, i32, i32, i32)>,
    ) -> Result<bool> {
        match condition {
            TriggerCondition::Timing { days_since, milestone } => {
                // Check timing-based triggers
                match milestone.as_str() {
                    "startup_created" => {
                        let days_old: Option<i64> = sqlx::query_scalar(
                            "SELECT EXTRACT(DAY FROM NOW() - created_at) FROM businesses WHERE id = $1"
                        )
                        .bind(business_id)
                        .fetch_optional(&self.db)
                        .await
                        .map_err(AppError::Database)?;
                        Ok(days_old.map(|d| d >= *days_since as i64).unwrap_or(false))
                    }
                    "bank_connected" => {
                        let days_since_connection: Option<i64> = sqlx::query_scalar(
                            "SELECT EXTRACT(DAY FROM NOW() - created_at) FROM bank_connections WHERE business_id = $1 AND status = 'connected'"
                        )
                        .bind(business_id)
                        .fetch_optional(&self.db)
                        .await
                        .map_err(AppError::Database)?;
                        Ok(days_since_connection.map(|d| d >= *days_since as i64).unwrap_or(false))
                    }
                    _ => Ok(false),
                }
            }
            TriggerCondition::HealthScore { component, threshold, operator } => {
                let Some((overall, compliance, revenue, market_fit, team, operations)) = health_score else {
                    return Ok(false);
                };

                let score = match component.as_str() {
                    "compliance" => compliance,
                    "revenue" => revenue,
                    "market_fit" => market_fit,
                    "team" => team,
                    "operations" => operations,
                    _ => overall,
                };

                match operator.as_str() {
                    "<" => Ok(score < *threshold),
                    ">" => Ok(score > *threshold),
                    "<=" => Ok(score <= *threshold),
                    ">=" => Ok(score >= *threshold),
                    _ => Ok(false),
                }
            }
            _ => Ok(false),
        }
    }

    /// Generate recommendation content using AI
    async fn generate_recommendation_content(&self, req: RecommendationContentRequest) -> Result<RecommendationContent> {
        let prompt = format!(
            r#"You are VentureMate's AI advisor. Generate a personalized recommendation.

Startup: {}
Industry: {}
Stage: {}
Trigger: {}

Generate:
1. Title: Short, action-oriented (max 50 chars)
2. Description: Why this matters, what to do (1-2 sentences)
3. Impact: What will improve if they do this
4. Call to Action: Specific next step

Be encouraging and specific. Avoid generic advice.

Return as JSON with keys: title, description, impact, call_to_action"#,
            req.startup_name, req.industry, req.business_stage, req.trigger_type
        );

        let response = self.ai_service.generate_text(
            "You are a helpful startup advisor. Provide actionable, specific advice.",
            &prompt,
            500
        ).await.map_err(|e| AppError::AiGeneration(e.to_string()))?;

        let content: RecommendationContent = serde_json::from_str(&response)
            .unwrap_or_else(|_| RecommendationContent {
                title: "Complete your startup setup".to_string(),
                description: "Take the next step to grow your business.".to_string(),
                impact: "Improved startup readiness".to_string(),
                call_to_action: "Get started now".to_string(),
            });

        Ok(content)
    }
}

impl RecommendationPriority {
    fn from_str(s: &str) -> Self {
        match s {
            "high" => Self::High,
            "low" => Self::Low,
            _ => Self::Medium,
        }
    }
}
