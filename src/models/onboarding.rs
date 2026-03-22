use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OnboardingSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub current_step: String,
    pub progress_percentage: i32,
    pub status: String,
    pub data: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct IdeaIntakeRequest {
    pub session_id: Uuid,
    #[validate(length(min = 10, message = "Business idea must be at least 10 characters"))]
    pub business_idea: String,
    pub problem_statement: Option<String>,
    pub target_customers: Option<String>,
    #[validate(length(equal = 2, message = "Country code must be 2 characters"))]
    pub country_code: String,
    pub city: Option<String>,
    pub founder_type: String,
    pub team_size: Option<i32>,
    pub has_cofounder: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FounderProfileRequest {
    pub session_id: Uuid,
    pub experience_level: Option<String>,
    pub background: Option<String>,
    pub skills: Option<Vec<String>>,
    pub availability: Option<String>,
    pub funding_preference: Option<String>,
    pub motivation: Option<String>,
    pub challenges: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BusinessDetailsRequest {
    pub session_id: Uuid,
    pub preferred_business_name: Option<String>,
    pub alternative_names: Option<Vec<String>>,
    pub business_model: Option<String>,
    pub revenue_streams: Option<Vec<String>>,
    pub initial_funding: Option<f64>,
    pub currency: Option<String>,
    pub timeline: Option<String>,
    pub legal_structure_preference: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReviewOnboardingRequest {
    pub session_id: Uuid,
    pub confirmed: bool,
    pub generate_options: GenerateOptions,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateOptions {
    pub business_plan: bool,
    pub branding_kit: bool,
    pub website: bool,
    pub pitch_deck: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct OnboardingSessionResponse {
    pub session_id: Uuid,
    pub current_step: String,
    pub progress_percentage: i32,
    pub steps: Vec<OnboardingStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OnboardingStep {
    pub id: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdeaIntakeResponse {
    pub session_id: Uuid,
    pub ai_analysis: AiAnalysis,
    pub next_step: String,
    pub progress_percentage: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnalysis {
    pub industry: String,
    pub sub_industry: String,
    pub market_size: String,
    pub complexity: String,
    pub estimated_launch_time: String,
    pub suggested_business_models: Vec<String>,
    pub viability_score: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OnboardingCompleteResponse {
    pub business_id: Uuid,
    pub generation_jobs: Vec<GenerationJobInfo>,
    pub estimated_completion: DateTime<Utc>,
    pub dashboard_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerationJobInfo {
    pub job_id: Uuid,
    pub job_type: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OnboardingStatusResponse {
    pub user_id: Uuid,
    pub steps_completed: Vec<String>,
    pub current_step: String,
    pub overall_progress: f32,
}

impl From<OnboardingSession> for OnboardingSessionResponse {
    fn from(session: OnboardingSession) -> Self {
        Self {
            session_id: session.id,
            current_step: session.current_step.clone(),
            progress_percentage: session.progress_percentage,
            steps: vec![
                OnboardingStep {
                    id: "idea_intake".to_string(),
                    name: "Business Idea".to_string(),
                    status: get_step_status(&session.current_step, "idea_intake"),
                },
                OnboardingStep {
                    id: "founder_profile".to_string(),
                    name: "Founder Profile".to_string(),
                    status: get_step_status(&session.current_step, "founder_profile"),
                },
                OnboardingStep {
                    id: "business_details".to_string(),
                    name: "Business Details".to_string(),
                    status: get_step_status(&session.current_step, "business_details"),
                },
                OnboardingStep {
                    id: "review".to_string(),
                    name: "Review & Generate".to_string(),
                    status: get_step_status(&session.current_step, "review"),
                },
            ],
        }
    }
}

fn get_step_status(current_step: &str, step_id: &str) -> String {
    let steps = vec!["idea_intake", "founder_profile", "business_details", "review"];
    let current_idx = steps.iter().position(|&s| s == current_step).unwrap_or(0);
    let step_idx = steps.iter().position(|&s| s == step_id).unwrap_or(0);
    
    if step_idx < current_idx {
        "completed".to_string()
    } else if step_idx == current_idx {
        "in_progress".to_string()
    } else {
        "pending".to_string()
    }
}
