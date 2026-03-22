//! Startup Stack Generator Models
//! Complete implementation per specification

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

// ============================================
// 1. STARTUP (Core Entity)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Startup {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub alternative_names: Value,
    pub tagline: Option<String>,
    pub elevator_pitch: Option<String>,
    pub mission_statement: Option<String>,
    pub vision_statement: Option<String>,
    pub industry: Option<String>,
    pub sub_industry: Option<String>,
    pub country: String,
    pub secondary_countries: Value,
    pub founder_type: String,
    pub business_stage: String,
    pub status: String,
    pub progress_percentage: i32,
    pub health_score: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub launched_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateStartupRequest {
    pub user_id: Uuid,
    pub name: String,
    pub alternative_names: Option<Vec<String>>,
    pub tagline: Option<String>,
    pub elevator_pitch: Option<String>,
    pub mission_statement: Option<String>,
    pub vision_statement: Option<String>,
    pub industry: Option<String>,
    pub sub_industry: Option<String>,
    pub country: String,
    pub secondary_countries: Option<Vec<String>>,
    pub founder_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStartupRequest {
    pub name: Option<String>,
    pub tagline: Option<String>,
    pub elevator_pitch: Option<String>,
    pub mission_statement: Option<String>,
    pub vision_statement: Option<String>,
    pub industry: Option<String>,
    pub sub_industry: Option<String>,
    pub business_stage: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StartupResponse {
    pub id: Uuid,
    pub name: String,
    pub tagline: Option<String>,
    pub industry: Option<String>,
    pub country: String,
    pub business_stage: String,
    pub status: String,
    pub progress_percentage: i32,
    pub health_score: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Startup> for StartupResponse {
    fn from(s: Startup) -> Self {
        Self {
            id: s.id,
            name: s.name,
            tagline: s.tagline,
            industry: s.industry,
            country: s.country,
            business_stage: s.business_stage,
            status: s.status,
            progress_percentage: s.progress_percentage,
            health_score: s.health_score,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

// ============================================
// 2. MILESTONE
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Milestone {
    pub id: Uuid,
    pub startup_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub order_sequence: i32,
    pub estimated_days: Option<i32>,
    pub estimated_cost: Option<f64>,
    pub status: String,
    pub completion_criteria: Value,
    pub depends_on_milestones: Value,
    pub assigned_to: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMilestoneRequest {
    pub status: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MilestoneResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub order_sequence: i32,
    pub estimated_days: Option<i32>,
    pub status: String,
    pub due_date: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub dependencies: Vec<Uuid>,
}

impl From<Milestone> for MilestoneResponse {
    fn from(m: Milestone) -> Self {
        let dependencies: Vec<Uuid> = serde_json::from_value(m.depends_on_milestones.clone())
            .unwrap_or_default();
        Self {
            id: m.id,
            title: m.title,
            description: m.description,
            category: m.category,
            order_sequence: m.order_sequence,
            estimated_days: m.estimated_days,
            status: m.status,
            due_date: m.due_date,
            completed_at: m.completed_at,
            dependencies,
        }
    }
}

// ============================================
// 3. REQUIRED APPROVAL
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RequiredApproval {
    pub id: Uuid,
    pub startup_id: Uuid,
    pub approval_type: String,
    pub name: String,
    pub issuing_authority: Option<String>,
    pub authority_website: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub priority: i32,
    pub estimated_days: Option<i32>,
    pub estimated_cost: Option<f64>,
    pub actual_cost: Option<f64>,
    pub documents_required: Value,
    pub documents_submitted: Value,
    pub submission_date: Option<DateTime<Utc>>,
    pub approval_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub reference_number: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateApprovalRequest {
    pub status: Option<String>,
    pub reference_number: Option<String>,
    pub submission_date: Option<DateTime<Utc>>,
    pub approval_date: Option<DateTime<Utc>>,
    pub actual_cost: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApprovalResponse {
    pub id: Uuid,
    pub name: String,
    pub approval_type: String,
    pub issuing_authority: Option<String>,
    pub status: String,
    pub priority: i32,
    pub estimated_days: Option<i32>,
    pub estimated_cost: Option<f64>,
    pub submission_date: Option<DateTime<Utc>>,
    pub approval_date: Option<DateTime<Utc>>,
    pub documents_required: Vec<String>,
    pub documents_submitted: Vec<String>,
}

impl From<RequiredApproval> for ApprovalResponse {
    fn from(a: RequiredApproval) -> Self {
        let docs_required: Vec<String> = serde_json::from_value(a.documents_required.clone())
            .unwrap_or_default();
        let docs_submitted: Vec<String> = serde_json::from_value(a.documents_submitted.clone())
            .unwrap_or_default();
        Self {
            id: a.id,
            name: a.name,
            approval_type: a.approval_type,
            issuing_authority: a.issuing_authority,
            status: a.status,
            priority: a.priority,
            estimated_days: a.estimated_days,
            estimated_cost: a.estimated_cost,
            submission_date: a.submission_date,
            approval_date: a.approval_date,
            documents_required: docs_required,
            documents_submitted: docs_submitted,
        }
    }
}

// ============================================
// 4. SUGGESTED SERVICE
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SuggestedService {
    pub id: Uuid,
    pub startup_id: Uuid,
    pub service_category: String,
    pub service_name: String,
    pub service_provider: Option<String>,
    pub description: Option<String>,
    pub features: Value,
    pub pricing_model: Option<String>,
    pub price_range: Option<String>,
    pub affiliate_link: Option<String>,
    pub website_url: Option<String>,
    pub integration_type: Option<String>,
    pub is_partner: bool,
    pub partnership_benefits: Option<String>,
    pub priority: i32,
    pub status: String,
    pub connected_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectServiceRequest {
    pub integration_data: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StartupServiceResponse {
    pub id: Uuid,
    pub service_category: String,
    pub service_name: String,
    pub service_provider: Option<String>,
    pub description: Option<String>,
    pub features: Vec<String>,
    pub pricing_model: Option<String>,
    pub price_range: Option<String>,
    pub website_url: Option<String>,
    pub is_partner: bool,
    pub partnership_benefits: Option<String>,
    pub priority: i32,
    pub status: String,
}

impl From<SuggestedService> for StartupServiceResponse {
    fn from(s: SuggestedService) -> Self {
        let features: Vec<String> = serde_json::from_value(s.features.clone())
            .unwrap_or_default();
        Self {
            id: s.id,
            service_category: s.service_category,
            service_name: s.service_name,
            service_provider: s.service_provider,
            description: s.description,
            features,
            pricing_model: s.pricing_model,
            price_range: s.price_range,
            website_url: s.website_url,
            is_partner: s.is_partner,
            partnership_benefits: s.partnership_benefits,
            priority: s.priority,
            status: s.status,
        }
    }
}

// ============================================
// 5. STARTUP DOCUMENT
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StartupDocument {
    pub id: Uuid,
    pub startup_id: Uuid,
    pub document_type: String,
    pub document_name: String,
    pub file_url: Option<String>,
    pub file_size: Option<i32>,
    pub version: i32,
    pub status: String,
    pub generated_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StartupDocumentResponse {
    pub id: Uuid,
    pub document_type: String,
    pub document_name: String,
    pub file_url: Option<String>,
    pub version: i32,
    pub status: String,
    pub generated_at: Option<DateTime<Utc>>,
}

// ============================================
// 6. STARTUP METRICS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StartupMetric {
    pub id: Uuid,
    pub startup_id: Uuid,
    pub metric_type: String,
    pub metric_value: f64,
    pub recorded_at: DateTime<Utc>,
    pub notes: Option<String>,
}

// ============================================
// 7. PROGRESS & HEALTH SCORE RESPONSES
// ============================================

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct StartupProgressResponse {
    pub startup_id: Uuid,
    pub overall_percentage: i32,
    pub completed_milestones: i64,
    pub total_milestones: i64,
    pub completed_approvals: i64,
    pub total_approvals: i64,
    pub connected_services: i64,
    pub total_services: i64,
    pub health_score: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StartupHealthScoreBreakdown {
    pub compliance_score: i32,
    pub milestone_progress: i32,
    pub document_completeness: i32,
    pub service_integration: i32,
    pub time_efficiency: i32,
    pub overall_health_score: i32,
}

// ============================================
// 8. DASHBOARD OVERVIEW
// ============================================

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct StartupOverview {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub progress_percentage: i32,
    pub health_score: Option<i32>,
    pub completed_milestones: Option<i64>,
    pub total_milestones: Option<i64>,
    pub completed_approvals: Option<i64>,
    pub total_approvals: Option<i64>,
    pub connected_services: Option<i64>,
    pub total_services: Option<i64>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct UpcomingDeadline {
    pub startup_id: Uuid,
    pub startup_name: String,
    pub milestone_id: Uuid,
    pub milestone_title: String,
    pub due_date: Option<DateTime<Utc>>,
    pub status: String,
    pub urgency: String,
}

// ============================================
// 9. DASHBOARD COMPONENTS
// ============================================

#[derive(Debug, Clone, Serialize)]
pub struct NextAction {
    pub action_id: String,
    pub action_type: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: i32,
    pub due_date: Option<String>,
    pub status: String,
    pub action_url: String,
    pub metadata: Option<String>,
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for NextAction {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> std::result::Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(Self {
            action_id: row.try_get("action_id")?,
            action_type: row.try_get("action_type")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            priority: row.try_get("priority")?,
            due_date: row.try_get("due_date")?,
            status: row.try_get("status")?,
            action_url: row.try_get("action_url")?,
            metadata: row.try_get("metadata")?,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StartupActivity {
    pub activity_type: String,
    pub description: String,
    pub occurred_at: DateTime<Utc>,
    pub metadata: Value,
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for StartupActivity {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> std::result::Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(Self {
            activity_type: row.try_get("activity_type")?,
            description: row.try_get("description")?,
            occurred_at: row.try_get("occurred_at")?,
            metadata: row.try_get("metadata")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct QuickStats {
    pub approvals_completed: Option<i64>,
    pub approvals_total: Option<i64>,
    pub documents_uploaded: Option<i64>,
    pub documents_total: Option<i64>,
    pub services_connected: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardData {
    pub startup: StartupOverview,
    pub health_score: i32,
    pub progress: StartupProgressResponse,
    pub next_actions: Vec<NextAction>,
    pub activity_feed: Vec<StartupActivity>,
    pub upcoming_deadlines: Vec<UpcomingDeadline>,
    pub quick_stats: QuickStats,
}

// ============================================
// 9. AI BLUEPRINT INPUT
// ============================================

#[derive(Debug, Clone, Deserialize)]
pub struct AiBlueprint {
    pub business_name: String,
    pub alternative_names: Vec<String>,
    pub tagline: Option<String>,
    pub elevator_pitch: Option<String>,
    pub mission_statement: Option<String>,
    pub vision_statement: Option<String>,
    pub industry: String,
    pub sub_industry: Option<String>,
    pub country: String,
    pub milestones: Vec<AiMilestone>,
    pub approvals: Vec<AiApproval>,
    pub services: Vec<AiServiceSuggestion>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiMilestone {
    pub title: String,
    pub description: String,
    pub category: String,
    pub estimated_days: i32,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiApproval {
    pub name: String,
    pub approval_type: String,
    pub issuing_authority: String,
    pub estimated_days: i32,
    pub documents_required: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiServiceSuggestion {
    pub category: String,
    pub name: String,
    pub provider: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateStartupStackResponse {
    pub startup_id: Uuid,
    pub milestones_created: usize,
    pub approvals_created: usize,
    pub services_suggested: usize,
    pub documents_queued: usize,
}
