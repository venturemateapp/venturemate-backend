use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{
    AiConversation, AiChatMessage, ChatMessageResponse, ChatWithAiResponse,
    CreateConversationRequest, SendMessageRequest, ConversationResponse,
    AiGeneratedContent, GenerateContentRequest, RegenerateContentRequest,
    GeneratedContentResponse, HealthScoreResponse, RecommendationResponse,
};
use crate::utils::{AppError, Result};

pub struct AiConversationService {
    db: PgPool,
}

impl AiConversationService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    // ============================================
    // CONVERSATION MANAGEMENT
    // ============================================

    pub async fn create_conversation(
        &self,
        user_id: Uuid,
        req: CreateConversationRequest,
    ) -> Result<ConversationResponse> {
        let conversation = sqlx::query_as::<_, AiConversation>(
            r#"
            INSERT INTO ai_conversations (user_id, business_id, session_type, status, context, metadata)
            VALUES ($1, $2, $3, 'active', '{}', $4)
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(req.business_id)
        .bind(&req.session_type)
        .bind(req.metadata.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.db)
        .await?;

        Ok(ConversationResponse {
            id: conversation.id,
            session_type: conversation.session_type,
            status: conversation.status,
            message_count: 0,
            created_at: conversation.created_at,
        })
    }

    pub async fn list_conversations(
        &self,
        user_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<ConversationResponse>> {
        let conversations = sqlx::query_as::<_, AiConversation>(
            r#"
            SELECT * FROM ai_conversations 
            WHERE user_id = $1 
            ORDER BY updated_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(user_id)
        .bind(per_page)
        .bind((page - 1) * per_page)
        .fetch_all(&self.db)
        .await?;

        let mut result = Vec::new();
        for conv in conversations {
            let count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM ai_chat_messages WHERE conversation_id = $1"
            )
            .bind(conv.id)
            .fetch_one(&self.db)
            .await?;

            result.push(ConversationResponse {
                id: conv.id,
                session_type: conv.session_type,
                status: conv.status,
                message_count: count,
                created_at: conv.created_at,
            });
        }

        Ok(result)
    }

    pub async fn get_conversation(&self, id: Uuid, user_id: Uuid) -> Result<ConversationResponse> {
        let conversation = sqlx::query_as::<_, AiConversation>(
            "SELECT * FROM ai_conversations WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Conversation not found".to_string()))?;

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM ai_chat_messages WHERE conversation_id = $1"
        )
        .bind(conversation.id)
        .fetch_one(&self.db)
        .await?;

        Ok(ConversationResponse {
            id: conversation.id,
            session_type: conversation.session_type,
            status: conversation.status,
            message_count: count,
            created_at: conversation.created_at,
        })
    }

    // ============================================
    // CHAT MESSAGES
    // ============================================

    pub async fn send_message(
        &self,
        conversation_id: Uuid,
        user_id: Uuid,
        req: SendMessageRequest,
    ) -> Result<ChatWithAiResponse> {
        // Verify conversation belongs to user
        let _conversation = sqlx::query_as::<_, AiConversation>(
            "SELECT * FROM ai_conversations WHERE id = $1 AND user_id = $2"
        )
        .bind(conversation_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Conversation not found".to_string()))?;

        // Save user message
        let user_msg = sqlx::query_as::<_, AiChatMessage>(
            r#"
            INSERT INTO ai_chat_messages (conversation_id, role, content, metadata)
            VALUES ($1, 'user', $2, '{}')
            RETURNING *
            "#
        )
        .bind(conversation_id)
        .bind(&req.content)
        .fetch_one(&self.db)
        .await?;

        // TODO: Integrate with Claude API for AI response
        let ai_content = format!("I received your message: '{}'", req.content);

        // Save AI response
        let ai_msg = sqlx::query_as::<_, AiChatMessage>(
            r#"
            INSERT INTO ai_chat_messages (conversation_id, role, content, ai_model, metadata)
            VALUES ($1, 'assistant', $2, 'claude-3-opus', '{}')
            RETURNING *
            "#
        )
        .bind(conversation_id)
        .bind(&ai_content)
        .fetch_one(&self.db)
        .await?;

        // Update conversation timestamp
        sqlx::query("UPDATE ai_conversations SET updated_at = NOW() WHERE id = $1")
            .bind(conversation_id)
            .execute(&self.db)
            .await?;

        Ok(ChatWithAiResponse {
            message: ChatMessageResponse {
                id: user_msg.id,
                role: user_msg.role,
                content: user_msg.content,
                ai_model: None,
                metadata: user_msg.metadata,
                created_at: user_msg.created_at,
            },
            ai_response: ChatMessageResponse {
                id: ai_msg.id,
                role: ai_msg.role,
                content: ai_msg.content,
                ai_model: ai_msg.ai_model,
                metadata: ai_msg.metadata,
                created_at: ai_msg.created_at,
            },
            actions: vec![],
            context_updates: req.context_updates.unwrap_or(serde_json::json!({})),
        })
    }

    pub async fn get_messages(
        &self,
        conversation_id: Uuid,
        user_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<ChatMessageResponse>> {
        // Verify conversation belongs to user
        let _conversation = sqlx::query_as::<_, AiConversation>(
            "SELECT * FROM ai_conversations WHERE id = $1 AND user_id = $2"
        )
        .bind(conversation_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Conversation not found".to_string()))?;

        let messages = sqlx::query_as::<_, AiChatMessage>(
            r#"
            SELECT * FROM ai_chat_messages 
            WHERE conversation_id = $1 
            ORDER BY created_at ASC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(conversation_id)
        .bind(per_page)
        .bind((page - 1) * per_page)
        .fetch_all(&self.db)
        .await?;

        Ok(messages.into_iter().map(|m| ChatMessageResponse {
            id: m.id,
            role: m.role,
            content: m.content,
            ai_model: m.ai_model,
            metadata: m.metadata,
            created_at: m.created_at,
        }).collect())
    }

    // ============================================
    // CONTENT GENERATION
    // ============================================

    pub async fn generate_content(
        &self,
        user_id: Uuid,
        req: GenerateContentRequest,
    ) -> Result<GeneratedContentResponse> {
        // Create content record
        let content = sqlx::query_as::<_, AiGeneratedContent>(
            r#"
            INSERT INTO ai_generated_content 
            (user_id, business_id, content_type, status, title, content, generation_params)
            VALUES ($1, $2, $3, 'generating', $4, '{}', $5)
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(req.business_id)
        .bind(&req.content_type)
        .bind(req.title.as_deref().unwrap_or("Untitled"))
        .bind(req.params)
        .fetch_one(&self.db)
        .await?;

        // TODO: Trigger async AI generation

        Ok(GeneratedContentResponse {
            id: content.id,
            content_type: content.content_type,
            status: content.status,
            title: content.title,
            content: content.content,
            version: content.version,
            created_at: content.created_at,
        })
    }

    pub async fn regenerate_content(
        &self,
        user_id: Uuid,
        content_id: Uuid,
        req: RegenerateContentRequest,
    ) -> Result<GeneratedContentResponse> {
        // Get original content
        let original = sqlx::query_as::<_, AiGeneratedContent>(
            "SELECT * FROM ai_generated_content WHERE id = $1 AND user_id = $2"
        )
        .bind(content_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Content not found".to_string()))?;

        // Create new version
        let new_content = sqlx::query_as::<_, AiGeneratedContent>(
            r#"
            INSERT INTO ai_generated_content 
            (user_id, business_id, content_type, status, title, content, generation_params, parent_version_id, version)
            VALUES ($1, $2, $3, 'generating', $4, '{}', $5, $6, $7)
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(original.business_id)
        .bind(&original.content_type)
        .bind(original.title.as_deref().unwrap_or("Untitled"))
        .bind(serde_json::json!({"regenerate_request": req.feedback}))
        .bind(content_id)
        .bind(original.version + 1)
        .fetch_one(&self.db)
        .await?;

        Ok(GeneratedContentResponse {
            id: new_content.id,
            content_type: new_content.content_type,
            status: new_content.status,
            title: new_content.title,
            content: new_content.content,
            version: new_content.version,
            created_at: new_content.created_at,
        })
    }

    pub async fn list_generated_content(
        &self,
        user_id: Uuid,
        business_id: Option<Uuid>,
        content_type: Option<String>,
    ) -> Result<Vec<GeneratedContentResponse>> {
        let mut query = String::from(
            "SELECT * FROM ai_generated_content WHERE user_id = $1"
        );
        
        if business_id.is_some() {
            query.push_str(" AND business_id = $2");
        }
        if content_type.is_some() {
            query.push_str(&format!(" AND content_type = ${}", if business_id.is_some() { 3 } else { 2 }));
        }
        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::query_as::<_, AiGeneratedContent>(&query).bind(user_id);
        
        if let Some(bid) = business_id {
            q = q.bind(bid);
        }
        if let Some(ct) = content_type {
            q = q.bind(ct);
        }

        let content = q.fetch_all(&self.db).await?;

        Ok(content.into_iter().map(|c| GeneratedContentResponse {
            id: c.id,
            content_type: c.content_type,
            status: c.status,
            title: c.title,
            content: c.content,
            version: c.version,
            created_at: c.created_at,
        }).collect())
    }

    // ============================================
    // HEALTH SCORE
    // ============================================

    pub async fn calculate_health_score(
        &self,
        _user_id: Uuid,
        _force_recalculate: bool,
    ) -> Result<HealthScoreResponse> {
        // TODO: Get user's business and calculate score
        // For now, return a mock score
        Ok(HealthScoreResponse {
            id: Uuid::new_v4(),
            business_id: Uuid::new_v4(),
            overall_score: 75,
            status: "healthy".to_string(),
            trend: "stable".to_string(),
            grade: Some("B".to_string()),
            components: crate::models::HealthScoreComponents {
                compliance: crate::models::ComponentScore {
                    score: 80, weight: 0.2, max_score: Some(100), grade: Some("B".to_string()), status: Some("good".to_string()), breakdown: None
                },
                revenue: crate::models::ComponentScore {
                    score: 70, weight: 0.2, max_score: Some(100), grade: Some("C".to_string()), status: Some("needs_work".to_string()), breakdown: None
                },
                market_fit: crate::models::ComponentScore {
                    score: 75, weight: 0.2, max_score: Some(100), grade: Some("B".to_string()), status: Some("good".to_string()), breakdown: None
                },
                team: crate::models::ComponentScore {
                    score: 85, weight: 0.2, max_score: Some(100), grade: Some("A".to_string()), status: Some("excellent".to_string()), breakdown: None
                },
                operations: crate::models::ComponentScore {
                    score: 70, weight: 0.2, max_score: Some(100), grade: Some("C".to_string()), status: Some("needs_work".to_string()), breakdown: None
                },
                funding_readiness: crate::models::ComponentScore {
                    score: 75, weight: 0.2, max_score: Some(100), grade: Some("B".to_string()), status: Some("good".to_string()), breakdown: None
                },
            },
            contributing_factors: crate::models::ContributingFactors {
                positive: vec!["Strong compliance record".to_string()],
                negative: vec!["Revenue growth needs attention".to_string()],
            },
            recommendations_count: 0,
            calculated_at: Utc::now(),
        })
    }

    // ============================================
    // RECOMMENDATIONS
    // ============================================

    pub async fn get_recommendations(&self, user_id: Uuid) -> Result<Vec<RecommendationResponse>> {
        let recommendations = sqlx::query_as::<_, crate::models::SmartRecommendation>(
            r#"
            SELECT * FROM smart_recommendations 
            WHERE user_id = $1 AND status IN ('unread', 'read')
            AND (valid_until IS NULL OR valid_until > NOW())
            ORDER BY 
                CASE priority 
                    WHEN 'urgent' THEN 1 
                    WHEN 'high' THEN 2 
                    WHEN 'medium' THEN 3 
                    ELSE 4 
                END,
                created_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(recommendations.into_iter().map(|r| RecommendationResponse {
            id: r.id,
            recommendation_type: r.recommendation_type,
            priority: r.priority.clone(),
            priority_label: r.priority,
            title: r.title,
            description: r.description,
            impact_description: r.impact_description,
            cta_text: r.cta_text,
            cta_link: r.cta_link,
            status: r.status,
            created_at: r.created_at,
        }).collect())
    }

    pub async fn dismiss_recommendation(
        &self,
        user_id: Uuid,
        recommendation_id: Uuid,
        reason: String,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE smart_recommendations 
            SET status = 'dismissed', dismissed_reason = $1, updated_at = NOW()
            WHERE id = $2 AND user_id = $3
            "#
        )
        .bind(reason)
        .bind(recommendation_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
