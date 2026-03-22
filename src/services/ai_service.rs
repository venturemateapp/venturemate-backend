use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::utils::{AppError, Result};

#[derive(Clone)]
pub struct AIService {
    client: Client,
    api_key: String,
    db: PgPool,
}

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    #[allow(dead_code)]
    id: String,
    content: Vec<ContentBlock>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[allow(dead_code)]
    #[serde(rename = "type")]
    block_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

impl AIService {
    pub fn new(api_key: impl Into<String>, db: PgPool) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            db,
        }
    }

    /// Generate business plan
    pub async fn generate_business_plan(
        &self,
        business_id: Uuid,
        user_id: Uuid,
        business_idea: &str,
        industry: &str,
        country: &str,
    ) -> Result<Value> {
        let system_prompt = r#"You are an expert business consultant with 20+ years experience helping startups succeed. Create comprehensive, actionable business plans tailored to the African market context.

Your business plans should be:
- Practical and actionable
- Based on real market data for the specified country
- Financially realistic
- Investor-ready
- formatted in Markdown with clear sections"#;

        let user_prompt = format!(
            r#"Create a comprehensive business plan for the following startup:

BUSINESS IDEA: {}
INDUSTRY: {}
TARGET COUNTRY: {}

Include these sections:
1. Executive Summary
2. Problem Statement
3. Solution
4. Market Analysis (TAM, SAM, SOM for {})
5. Business Model & Revenue Streams
6. Competitive Analysis
7. Go-to-Market Strategy
8. Financial Projections (3-year)
9. Team Requirements
10. Risk Analysis & Mitigation

Format your response in Markdown."#,
            business_idea, industry, country, country
        );

        let response = self
            .call_claude(Some(system_prompt), &user_prompt, 4000)
            .await?;

        // Parse and structure the content
        let structured_content = self.parse_business_plan(&response.content[0].text);

        // Save to database
        self.save_generated_document(
            business_id,
            user_id,
            "business_plan",
            &structured_content,
            "claude-3-5-sonnet-20241022",
            response.usage.input_tokens + response.usage.output_tokens,
        )
        .await?;

        Ok(structured_content)
    }

    /// Generate pitch deck content
    pub async fn generate_pitch_deck(
        &self,
        business_id: Uuid,
        user_id: Uuid,
        business_name: &str,
        tagline: &str,
        industry: &str,
        stage: &str,
    ) -> Result<Value> {
        let system_prompt = r#"You are an expert pitch deck creator who has helped startups raise over $100M. You create compelling, investor-focused pitch decks following the Sequoia format.

Each slide should have:
- A compelling headline
- 3-5 bullet points maximum
- Key metrics when relevant"#;

        let user_prompt = format!(
            r#"Create a pitch deck outline for:

BUSINESS: {}
TAGLINE: {}
INDUSTRY: {}
STAGE: {}

Create content for these 12 slides:
1. Title Slide
2. Problem
3. Solution
4. Product Demo
5. Traction
6. Market Opportunity
7. Business Model
8. Competition
9. Go-to-Market
10. Team
11. Financials
12. The Ask

For each slide provide:
- Title
- Key message (1-2 sentences)
- 3-5 bullet points
- Visual suggestion

Format as structured JSON."#,
            business_name, tagline, industry, stage
        );

        let response = self
            .call_claude(Some(system_prompt), &user_prompt, 3000)
            .await?;

        // Try to parse as JSON, fallback to text
        let content = match serde_json::from_str::<Value>(&response.content[0].text) {
            Ok(json) => json,
            Err(_) => json!({"content": response.content[0].text}),
        };

        self.save_generated_document(
            business_id,
            user_id,
            "pitch_deck",
            &content,
            "claude-3-5-sonnet-20241022",
            response.usage.input_tokens + response.usage.output_tokens,
        )
        .await?;

        Ok(content)
    }

    /// Analyze business idea and return structured analysis
    pub async fn analyze_business_idea(
        &self,
        idea: &str,
        problem: Option<&str>,
        target: Option<&str>,
    ) -> Result<Value> {
        let prompt = format!(
            r#"Analyze this business idea and return ONLY a JSON object:

IDEA: {}
PROBLEM: {}
TARGET CUSTOMERS: {}

Respond with ONLY this JSON structure (no markdown, no explanation):
{{
    "industry": "Primary industry category",
    "sub_industry": "More specific category",
    "market_size": "Brief market size estimate",
    "complexity": "low|medium|high",
    "estimated_launch_time": "X weeks/months",
    "suggested_business_models": ["model1", "model2", "model3"],
    "viability_score": 1-10,
    "key_insights": "2-3 key insights about this idea"
}}"#,
            idea,
            problem.unwrap_or("Not specified"),
            target.unwrap_or("Not specified")
        );

        let response = self.call_claude(None, &prompt, 1000).await?;
        
        // Extract JSON from response
        let text = &response.content[0].text;
        let json_str = if text.contains("```json") {
            text.split("```json").nth(1).unwrap_or(text).split("```").next().unwrap_or(text)
        } else if text.contains("```") {
            text.split("```").nth(1).unwrap_or(text).split("```").next().unwrap_or(text)
        } else {
            text
        };

        let analysis: Value = serde_json::from_str(json_str.trim())
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse AI response: {}", e)))?;

        Ok(analysis)
    }

    /// Generate brand color palette
    pub async fn generate_color_palette(
        &self,
        business_name: &str,
        industry: &str,
        mood: Option<&str>,
        base_color: Option<&str>,
    ) -> Result<Value> {
        let mood_str = mood.unwrap_or("professional, trustworthy");
        let base_str = base_color.map(|c| format!("Base the palette around {}.", c)).unwrap_or_default();
        
        let prompt = format!(
            "Generate a professional color palette for '{}' a {} business with a {} mood. {}

Return ONLY a JSON object with these exact keys and hex color codes:
{{{{
    \"primary\": \"#XXXXXX\",
    \"secondary\": \"#XXXXXX\", 
    \"accent\": \"#XXXXXX\",
    \"neutral\": \"#XXXXXX\",
    \"background\": \"#XXXXXX\",
    \"text\": \"#XXXXXX\",
    \"success\": \"#XXXXXX\",
    \"warning\": \"#XXXXXX\",
    \"error\": \"#XXXXXX\",
    \"ai_recommendation\": \"Brief explanation of why these colors work\"
}}}}",
            business_name, industry, mood_str, base_str
        );

        let response = self.call_claude(None, &prompt, 500).await?;
        
        let text = &response.content[0].text;
        let json_str = if text.contains("```json") {
            text.split("```json").nth(1).unwrap_or(text).split("```").next().unwrap_or(text)
        } else if text.contains("```") {
            text.split("```").nth(1).unwrap_or(text).split("```").next().unwrap_or(text)
        } else {
            text
        };

        let palette: Value = serde_json::from_str(json_str.trim())
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse color palette: {}", e)))?;

        Ok(palette)
    }

    /// Generate logo prompt for DALL-E
    pub async fn generate_logo_prompt(
        &self,
        business_name: &str,
        industry: &str,
        style: &str,
        colors: &[String],
    ) -> Result<String> {
        let prompt = format!(
            r#"Create a professional logo design prompt for '{}' a {} business.
Style: {}
Color palette: {}

Generate a detailed prompt suitable for an AI image generator (like DALL-E). The prompt should:
- Describe a clean, modern logo
- Include style, colors, and mood
- Be specific about composition
- Request transparent or white background
- Specify no text in the image

Return ONLY the prompt text, nothing else."#,
            business_name,
            industry,
            style,
            colors.join(", ")
        );

        let response = self.call_claude(None, &prompt, 500).await?;
        Ok(response.content[0].text.trim().to_string())
    }

    /// Call Claude API
    async fn call_claude(
        &self,
        system: Option<&str>,
        user_message: &str,
        max_tokens: u32,
    ) -> Result<ClaudeResponse> {
        let request = ClaudeRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: user_message.to_string(),
            }],
            system: system.map(|s| s.to_string()),
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("AI request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalApi(format!("AI API error: {}", error_text)));
        }

        let claude_response: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse AI response: {}", e)))?;

        Ok(claude_response)
    }

    /// Parse business plan markdown into structured JSON
    fn parse_business_plan(&self, content: &str) -> Value {
        let mut sections = serde_json::Map::new();
        let mut current_section = String::new();
        let mut current_content = String::new();

        for line in content.lines() {
            if line.starts_with("# ") || line.starts_with("## ") {
                if !current_section.is_empty() {
                    sections.insert(
                        current_section.to_lowercase().replace(" ", "_"),
                        json!(current_content.trim()),
                    );
                }
                current_section = line.trim_start_matches("# ").trim_start_matches("## ").to_string();
                current_content = String::new();
            } else {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }

        if !current_section.is_empty() {
            sections.insert(
                current_section.to_lowercase().replace(" ", "_"),
                json!(current_content.trim()),
            );
        }

        json!(sections)
    }

    /// Save generated document to database
    async fn save_generated_document(
        &self,
        business_id: Uuid,
        _user_id: Uuid,
        document_type: &str,
        content: &Value,
        ai_model: &str,
        token_usage: u32,
    ) -> Result<()> {
        // Get next version number
        let version = sqlx::query_scalar::<_, i32>(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM generated_documents WHERE business_id = $1 AND document_type = $2"
        )
        .bind(business_id)
        .bind(document_type)
        .fetch_one(&self.db)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO generated_documents (business_id, document_type, version, content, ai_model, token_usage)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(business_id)
        .bind(document_type)
        .bind(version)
        .bind(content)
        .bind(ai_model)
        .bind(token_usage as i32)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Generate text using Claude API (generic method)
    pub async fn generate_text(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: u32,
        temperature: Option<f32>,
    ) -> Result<String> {
        let response = self
            .call_claude(Some(system_prompt), user_prompt, max_tokens)
            .await?;

        Ok(response.content[0].text.clone())
    }
}
