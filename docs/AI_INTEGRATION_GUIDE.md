# VentureMate - AI Integration Guide

> Complete guide to integrating AI services for business generation

## 📋 Table of Contents

1. [Overview](#1-overview)
2. [Setting Up AI Providers](#2-setting-up-ai-providers)
3. [Prompt Engineering](#3-prompt-engineering)
4. [Business Plan Generation](#4-business-plan-generation)
5. [Logo & Branding Generation](#5-logo--branding-generation)
6. [Website Generation](#6-website-generation)
7. [Content Generation](#7-content-generation)
8. [Cost Optimization](#8-cost-optimization)
9. [Error Handling & Retries](#9-error-handling--retries)

---

## 1. Overview

### AI Services Used

| Service | Provider | Use Case | Cost |
|---------|----------|----------|------|
| **LLM** | Claude 3.5 Sonnet / GPT-4 | Business plans, pitch decks, content | ~$3-5 per generation |
| **Image Gen** | DALL-E 3 / Replicate | Logos, brand assets | ~$0.04-0.08 per image |
| **Embeddings** | OpenAI Ada-002 | Similarity search | ~$0.10 per 1M tokens |
| **Speech-to-Text** | Whisper API | Voice note processing | ~$0.006 per minute |

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     AI GENERATION FLOW                          │
└─────────────────────────────────────────────────────────────────┘

User Request
     │
     ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Validate  │────►│  Enrich     │────►│  Assemble   │
│   Input     │     │  Context    │     │  Prompt     │
└─────────────┘     └─────────────┘     └──────┬──────┘
                                               │
                                               ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Store     │◄────│   Parse     │◄────│   Call AI   │
│   Result    │     │   Response  │     │   API       │
└─────────────┘     └─────────────┘     └─────────────┘
```

---

## 2. Setting Up AI Providers

### 2.1 Claude (Anthropic) - Recommended for Text

**Sign Up:** https://console.anthropic.com/

**Environment Variables:**
```bash
# .env
ANTHROPIC_API_KEY=sk-ant-your-key-here
ANTHROPIC_BASE_URL=https://api.anthropic.com/v1/messages
```

**Rust Implementation:**

```rust
// src/integrations/anthropic.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct AnthropicClient {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    temperature: f32,
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
pub struct ClaudeResponse {
    pub id: String,
    pub content: Vec<ContentBlock>,
    pub usage: Usage,
}

#[derive(Debug, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl AnthropicClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.anthropic.com/v1/messages".to_string(),
        }
    }

    pub async fn complete(
        &self,
        messages: Vec<(String, String)>, // (role, content)
        system_prompt: Option<String>,
        max_tokens: u32,
    ) -> Result<ClaudeResponse, reqwest::Error> {
        let claude_messages: Vec<ClaudeMessage> = messages
            .into_iter()
            .map(|(role, content)| ClaudeMessage { role, content })
            .collect();

        let request = ClaudeRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens,
            temperature: 0.7,
            messages: claude_messages,
            system: system_prompt,
        };

        let response = self
            .client
            .post(&self.base_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?
            .json::<ClaudeResponse>()
            .await?;

        Ok(response)
    }
}
```

### 2.2 OpenAI - Alternative for Text

**Sign Up:** https://platform.openai.com/

**Environment Variables:**
```bash
OPENAI_API_KEY=sk-your-key-here
```

**Rust Implementation:**

```rust
// src/integrations/openai.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenAIClient {
    client: Client,
    api_key: String,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIResponse {
    pub choices: Vec<Choice>,
    pub usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn complete(
        &self,
        messages: Vec<(String, String)>,
        max_tokens: u32,
    ) -> Result<OpenAIResponse, reqwest::Error> {
        let openai_messages: Vec<OpenAIMessage> = messages
            .into_iter()
            .map(|(role, content)| OpenAIMessage { role, content })
            .collect();

        let request = OpenAIRequest {
            model: "gpt-4".to_string(),
            messages: openai_messages,
            max_tokens,
            temperature: 0.7,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?
            .json::<OpenAIResponse>()
            .await?;

        Ok(response)
    }
}
```

### 2.3 DALL-E / Replicate - Image Generation

**For Logos:**

```rust
// src/integrations/dalle.rs
pub async fn generate_logo(
    &self,
    business_name: &str,
    industry: &str,
    style: &str,
) -> Result<String, reqwest::Error> {
    let prompt = format!(
        "Professional logo design for '{}' a {} business. \
         Style: {}. Clean, modern, suitable for business use. \
         No text in the image. Simple geometric shapes.",
        business_name, industry, style
    );

    let request = serde_json::json!({
        "model": "dall-e-3",
        "prompt": prompt,
        "size": "1024x1024",
        "quality": "standard",
        "n": 1,
    });

    let response = self
        .client
        .post("https://api.openai.com/v1/images/generations")
        .header("authorization", format!("Bearer {}", self.api_key))
        .json(&request)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(response["data"][0]["url"].as_str().unwrap().to_string())
}
```

---

## 3. Prompt Engineering

### 3.1 Business Plan Prompt Template

```rust
pub const BUSINESS_PLAN_SYSTEM_PROMPT: &str = r#"You are an expert business consultant with 20 years of experience helping startups succeed. You specialize in creating comprehensive, actionable business plans for African entrepreneurs.

Your business plans are:
- Practical and actionable
- Based on real market data
- Tailored to local context
- Financially realistic
- Investor-ready

Respond in Markdown format with clear sections."#;

pub const BUSINESS_PLAN_TEMPLATE: &str = r#"Create a comprehensive business plan for the following startup:

BUSINESS IDEA: {business_idea}
INDUSTRY: {industry}
TARGET COUNTRY: {country}
TARGET CUSTOMERS: {target_customers}
STAGE: {stage}

Include these sections:

# Executive Summary
Brief overview of the business opportunity

# Problem Statement
Clear description of the problem being solved

# Solution
How the business solves the problem

# Market Analysis
- Total Addressable Market (TAM) for {country}
- Serviceable Addressable Market (SAM)
- Serviceable Obtainable Market (SOM)
- Market trends and growth

# Business Model
- Revenue streams
- Pricing strategy
- Unit economics

# Competitive Analysis
- Direct competitors
- Indirect competitors
- Competitive advantages

# Go-to-Market Strategy
- Customer acquisition channels
- Marketing approach
- Partnerships

# Financial Projections
3-year projections including:
- Revenue forecast
- Cost structure
- Break-even analysis
- Key metrics

# Team & Operations
- Key roles needed
- Organizational structure
- Operational plan

# Risk Analysis & Mitigation
- Key risks
- Mitigation strategies

# Funding Requirements
- Amount needed
- Use of funds
- Milestones
"#;
```

### 3.2 Pitch Deck Prompt Template

```rust
pub const PITCH_DECK_SYSTEM_PROMPT: &str = r#"You are an expert pitch deck creator who has helped startups raise over $100M. You create compelling, investor-focused pitch decks that tell a story.

Your pitch decks follow the Sequoia format:
1. Problem
2. Solution
3. Why Now
4. Market Size
5. Competition
6. Product
7. Business Model
8. Team
9. Financials
10. The Ask

Each slide should have:
- A compelling headline
- Bullet points (max 5 per slide)
- Key metrics when relevant"#;

pub const PITCH_DECK_TEMPLATE: &str = r#"Create a pitch deck outline for:

BUSINESS: {business_name}
TAGLINE: {tagline}
INDUSTRY: {industry}
STAGE: {stage}

For each slide, provide:
1. Slide title
2. Key message (1-2 sentences)
3. Bullet points (3-5 points)
4. Visual suggestion

SLIDES TO CREATE:
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
"#;
```

### 3.3 Prompt Variables

```rust
use handlebars::Handlebars;
use serde_json::json;

pub struct PromptEngine {
    registry: Handlebars<'static>,
}

impl PromptEngine {
    pub fn new() -> Self {
        let mut registry = Handlebars::new();
        
        // Register templates
        registry.register_template_string("business_plan", BUSINESS_PLAN_TEMPLATE).unwrap();
        registry.register_template_string("pitch_deck", PITCH_DECK_TEMPLATE).unwrap();
        
        Self { registry }
    }

    pub fn render_business_plan_prompt(&self, params: BusinessPlanParams) -> String {
        let data = json!({
            "business_idea": params.business_idea,
            "industry": params.industry,
            "country": params.country,
            "target_customers": params.target_customers,
            "stage": params.stage,
        });
        
        self.registry.render("business_plan", &data).unwrap()
    }
}

pub struct BusinessPlanParams {
    pub business_idea: String,
    pub industry: String,
    pub country: String,
    pub target_customers: String,
    pub stage: String,
}
```

---

## 4. Business Plan Generation

### 4.1 Complete Implementation

```rust
// src/services/business_plan_generator.rs
use crate::integrations::anthropic::{AnthropicClient, ClaudeResponse};
use crate::models::ai_generation::{AiGenerationJob, GeneratedDocument};
use crate::utils::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

pub struct BusinessPlanGenerator {
    ai_client: AnthropicClient,
    db: PgPool,
    prompt_engine: PromptEngine,
}

impl BusinessPlanGenerator {
    pub fn new(ai_client: AnthropicClient, db: PgPool) -> Self {
        Self {
            ai_client,
            db,
            prompt_engine: PromptEngine::new(),
        }
    }

    pub async fn generate(
        &self,
        business_id: Uuid,
        user_id: Uuid,
        params: BusinessPlanParams,
    ) -> Result<Uuid, AppError> {
        // 1. Create job record
        let job_id = self.create_job(business_id, user_id, "business_plan").await?;

        // 2. Start generation in background
        let ai_client = self.ai_client.clone();
        let db = self.db.clone();
        let prompt_engine = self.prompt_engine.clone();
        let params_clone = params.clone();

        tokio::spawn(async move {
            let result = Self::execute_generation(
                ai_client,
                db.clone(),
                prompt_engine,
                job_id,
                params_clone,
            ).await;

            if let Err(e) = result {
                // Log error and update job status
                let _ = sqlx::query(
                    "UPDATE ai_generation_jobs SET status = 'failed', error_message = $1 WHERE id = $2"
                )
                .bind(e.to_string())
                .bind(job_id)
                .execute(&db)
                .await;
            }
        });

        Ok(job_id)
    }

    async fn execute_generation(
        ai_client: AnthropicClient,
        db: PgPool,
        prompt_engine: PromptEngine,
        job_id: Uuid,
        params: BusinessPlanParams,
    ) -> Result<(), AppError> {
        // Update status to processing
        sqlx::query("UPDATE ai_generation_jobs SET status = 'processing', started_at = NOW() WHERE id = $1")
            .bind(job_id)
            .execute(&db)
            .await?;

        // Generate prompt
        let user_prompt = prompt_engine.render_business_plan_prompt(params);

        // Call AI
        let response = ai_client
            .complete(
                vec![("user".to_string(), user_prompt)],
                Some(BUSINESS_PLAN_SYSTEM_PROMPT.to_string()),
                4000,
            )
            .await
            .map_err(|e| AppError::Internal(format!("AI request failed: {}", e)))?;

        let content = response
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| AppError::Internal("Empty AI response".to_string()))?;

        // Parse and structure the content
        let structured_content = parse_business_plan(&content)?;

        // Save generated document
        let doc_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO generated_documents 
            (id, business_id, job_id, document_type, content, ai_model, token_usage)
            VALUES ($1, (SELECT business_id FROM ai_generation_jobs WHERE id = $2), $2, 'business_plan', $3, $4, $5)
            "#
        )
        .bind(doc_id)
        .bind(job_id)
        .bind(&structured_content)
        .bind("claude-3-5-sonnet-20241022")
        .bind((response.usage.input_tokens + response.usage.output_tokens) as i32)
        .execute(&db)
        .await?;

        // Update job status
        sqlx::query(
            r#"
            UPDATE ai_generation_jobs 
            SET status = 'completed', 
                completed_at = NOW(),
                result = $1,
                token_usage = $2
            WHERE id = $3
            "#
        )
        .bind(&structured_content)
        .bind((response.usage.input_tokens + response.usage.output_tokens) as i32)
        .bind(job_id)
        .execute(&db)
        .await?;

        Ok(())
    }

    async fn create_job(
        &self,
        business_id: Uuid,
        user_id: Uuid,
        job_type: &str,
    ) -> Result<Uuid, AppError> {
        let job_id = Uuid::new_v4();
        
        sqlx::query(
            r#"
            INSERT INTO ai_generation_jobs (id, business_id, user_id, job_type, status, progress)
            VALUES ($1, $2, $3, $4, 'queued', 0)
            "#
        )
        .bind(job_id)
        .bind(business_id)
        .bind(user_id)
        .bind(job_type)
        .execute(&self.db)
        .await?;

        Ok(job_id)
    }
}

fn parse_business_plan(content: &str) -> Result<serde_json::Value, AppError> {
    // Parse the markdown content into structured JSON
    // This is a simplified version - you might want more sophisticated parsing
    
    let sections: Vec<&str> = content.split("# ").collect();
    
    let mut structured = serde_json::Map::new();
    
    for section in sections.iter().skip(1) {
        let lines: Vec<&str> = section.lines().collect();
        if let Some(title) = lines.first() {
            let title = title.trim();
            let content = lines[1..].join("\n").trim().to_string();
            structured.insert(title.to_lowercase().replace(" ", "_"), serde_json::json!(content));
        }
    }
    
    Ok(serde_json::Value::Object(structured))
}
```

---

## 5. Logo & Branding Generation

### 5.1 Logo Generation Service

```rust
// src/services/logo_generator.rs
use crate::integrations::dalle::DalleClient;
use crate::utils::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

pub struct LogoGenerator {
    image_client: DalleClient,
    db: PgPool,
}

impl LogoGenerator {
    pub fn new(image_client: DalleClient, db: PgPool) -> Self {
        Self { image_client, db }
    }

    pub async fn generate_options(
        &self,
        business_id: Uuid,
        business_name: &str,
        industry: &str,
        preferences: LogoPreferences,
    ) -> Result<Vec<Uuid>, AppError> {
        let mut logo_ids = Vec::new();
        
        // Generate multiple variations
        let styles = vec!["minimalist", "modern", "geometric", "abstract"];
        
        for style in styles.iter().take(preferences.variations_count as usize) {
            let prompt = format!(
                "Professional {} logo for '{}' a {} company. \
                 Color palette: {}. Clean, scalable vector style. \
                 Suitable for app icon, website, and business cards. \
                 White or transparent background.",
                style,
                business_name,
                industry,
                preferences.color_hints.join(", ")
            );

            // Generate image
            let image_url = self.image_client.generate_image(&prompt).await?;
            
            // Download and store
            let logo_id = self.store_logo(business_id, &image_url, style).await?;
            logo_ids.push(logo_id);
        }

        Ok(logo_ids)
    }

    async fn store_logo(
        &self,
        business_id: Uuid,
        image_url: &str,
        style: &str,
    ) -> Result<Uuid, AppError> {
        let logo_id = Uuid::new_v4();
        
        // Download image
        let image_data = reqwest::get(image_url).await?.bytes().await?;
        
        // Upload to S3/Storage
        let storage_url = upload_to_storage(&image_data, &format!("logos/{}", logo_id)).await?;

        sqlx::query(
            r#"
            INSERT INTO brand_assets (id, business_id, asset_type, file_url, variant, is_active)
            VALUES ($1, $2, 'logo', $3, $4, true)
            "#
        )
        .bind(logo_id)
        .bind(business_id)
        .bind(&storage_url)
        .bind(style)
        .execute(&self.db)
        .await?;

        Ok(logo_id)
    }
}

pub struct LogoPreferences {
    pub variations_count: i32,
    pub color_hints: Vec<String>,
    pub style_preference: Option<String>,
}
```

### 5.2 Color Palette Generation

```rust
pub async fn generate_color_palette(
    &self,
    industry: &str,
    mood: &str,
    base_color: Option<String>,
) -> Result<ColorPalette, AppError> {
    let prompt = format!(
        "Generate a professional color palette for a {} business with a {} mood. \
         {} \
         Return ONLY a JSON object with these exact keys: \
         primary, secondary, accent, neutral, background, text, success, warning, error. \
         Each value should be a hex color code.",
        industry,
        mood,
        base_color.map(|c| format!("Base the palette around {}.", c)).unwrap_or_default()
    );

    let response = self.ai_client
        .complete(vec![("user".to_string(), prompt)], None, 500)
        .await?;

    let content = response.content.first().unwrap().text.clone();
    
    // Parse JSON from response
    let palette: ColorPalette = serde_json::from_str(&content)
        .map_err(|e| AppError::Internal(format!("Failed to parse color palette: {}", e)))?;

    Ok(palette)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ColorPalette {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub neutral: String,
    pub background: String,
    pub text: String,
    pub success: String,
    pub warning: String,
    pub error: String,
}
```

---

## 6. Website Generation

### 6.1 Template-Based Generation

```rust
// src/services/website_generator.rs
pub struct WebsiteGenerator {
    template_engine: TemplateEngine,
    db: PgPool,
}

impl WebsiteGenerator {
    pub async fn generate(
        &self,
        business_id: Uuid,
        template_code: &str,
        business_data: BusinessData,
    ) -> Result<String, AppError> {
        // 1. Load template
        let template = self.load_template(template_code).await?;
        
        // 2. Generate content sections using AI
        let sections = self.generate_sections(&business_data).await?;
        
        // 3. Render template with data
        let html = self.template_engine.render(&template, &json!({
            "business_name": business_data.name,
            "tagline": business_data.tagline,
            "description": business_data.description,
            "primary_color": business_data.brand_colors.primary,
            "sections": sections,
        }))?;

        // 4. Store generated website
        self.store_website(business_id, &html).await?;

        Ok(html)
    }

    async fn generate_sections(
        &self,
        business_data: &BusinessData,
    ) -> Result<Vec<Section>, AppError> {
        let prompt = format!(
            "Generate website content sections for {}. \
             Industry: {}. Description: {}. \
             Create content for: Hero, About, Features, Contact sections. \
             Return as JSON array with 'id', 'title', 'content' for each.",
            business_data.name,
            business_data.industry,
            business_data.description
        );

        let response = self.ai_client.complete(
            vec![("user".to_string(), prompt)],
            Some("You are a professional web copywriter. Create compelling, concise website content.".to_string()),
            2000,
        ).await?;

        let sections: Vec<Section> = serde_json::from_str(&response.content[0].text)?;
        Ok(sections)
    }
}
```

---

## 7. Content Generation

### 7.1 Social Media Posts

```rust
pub async fn generate_social_content(
    &self,
    business_name: &str,
    industry: &str,
    platforms: Vec<String>,
    posts_per_platform: i32,
) -> Result<HashMap<String, Vec<String>>, AppError> {
    let mut content = HashMap::new();

    for platform in platforms {
        let prompt = format!(
            "Generate {} {} posts for '{}' a {} business. \
             Tone: Professional yet approachable. \
             Target audience: African entrepreneurs and small business owners.",
            posts_per_platform, platform, business_name, industry
        );

        let response = self.ai_client.complete(
            vec![("user".to_string(), prompt)],
            None,
            1500,
        ).await?;

        let posts: Vec<String> = response.content[0].text
            .split("\n\n")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        content.insert(platform, posts);
    }

    Ok(content)
}
```

---

## 8. Cost Optimization

### 8.1 Caching Strategy

```rust
use redis::AsyncCommands;

pub struct AICache {
    redis: redis::aio::MultiplexedConnection,
}

impl AICache {
    pub async fn get_cached_response(
        &self,
        prompt_hash: &str,
    ) -> Result<Option<String>, redis::RedisError> {
        let mut conn = self.redis.clone();
        conn.get(format!("ai:cache:{}", prompt_hash)).await
    }

    pub async fn cache_response(
        &self,
        prompt_hash: &str,
        response: &str,
        ttl_seconds: u64,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.redis.clone();
        conn.set_ex(format!("ai:cache:{}", prompt_hash), response, ttl_seconds).await
    }
}

// Usage in generation
let prompt_hash = hash_prompt(&prompt);
if let Some(cached) = cache.get_cached_response(&prompt_hash).await? {
    return Ok(cached);
}

let response = ai_client.complete(...).await?;
cache.cache_response(&prompt_hash, &response, 3600).await?;
```

### 8.2 Token Optimization

```rust
pub fn optimize_prompt(prompt: &str) -> String {
    prompt
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn truncate_context(context: &str, max_tokens: usize) -> String {
    // Rough estimate: 1 token ≈ 4 characters
    let max_chars = max_tokens * 4;
    if context.len() > max_chars {
        format!("{}...", &context[..max_chars])
    } else {
        context.to_string()
    }
}
```

---

## 9. Error Handling & Retries

### 9.1 Retry Logic

```rust
use tokio::time::{sleep, Duration};

pub async fn with_exponential_retry<F, Fut, T>(
    operation: F,
    max_retries: u32,
) -> Result<T, AppError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, reqwest::Error>>,
{
    let mut retries = 0;
    let mut delay = Duration::from_secs(1);

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                retries += 1;
                
                // Check if error is retryable
                if let Some(status) = e.status() {
                    if status.as_u16() == 429 || status.as_u16() >= 500 {
                        tracing::warn!("Retryable error ({}), attempt {}/{}", status, retries, max_retries);
                        sleep(delay).await;
                        delay *= 2; // Exponential backoff
                        continue;
                    }
                }
                
                return Err(AppError::Internal(e.to_string()));
            }
            Err(e) => return Err(AppError::Internal(e.to_string())),
        }
    }
}
```

### 9.2 Fallback Strategy

```rust
pub struct AIFailover {
    primary: AnthropicClient,
    fallback: OpenAIClient,
}

impl AIFailover {
    pub async fn complete(&self, messages: Vec<(String, String)>) -> Result<String, AppError> {
        // Try primary first
        match self.primary.complete(messages.clone(), None, 4000).await {
            Ok(response) => Ok(response.content[0].text.clone()),
            Err(_) => {
                tracing::warn!("Primary AI failed, falling back to backup");
                // Try fallback
                let response = self.fallback.complete(messages, 4000).await
                    .map_err(|e| AppError::Internal(format!("Both AI providers failed: {}", e)))?;
                Ok(response.choices[0].message.content.clone())
            }
        }
    }
}
```

---

## Quick Reference: AI Costs

| Operation | Provider | Input Tokens | Output Tokens | Cost (USD) |
|-----------|----------|--------------|---------------|------------|
| Business Plan | Claude 3.5 | 500 | 2,500 | $0.052 |
| Pitch Deck | Claude 3.5 | 400 | 2,000 | $0.042 |
| Logo | DALL-E 3 | - | - | $0.040 |
| Color Palette | Claude 3.5 | 100 | 200 | $0.005 |
| Website Content | Claude 3.5 | 300 | 1,500 | $0.030 |
| Social Posts (10) | Claude 3.5 | 200 | 800 | $0.018 |

**Monthly Estimate (1000 generations):**
- Text AI: ~$150
- Image AI: ~$200
- **Total: ~$350/month**

---

**Document Version**: 1.0.0  
**Last Updated**: 2025-03-20
