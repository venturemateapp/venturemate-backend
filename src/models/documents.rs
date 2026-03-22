//! Document Generation & Data Room Models
//! 
//! Models for business plans, pitch decks, one-pagers,
//! and secure investor data rooms.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// =============================================================================
// Generated Documents
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GeneratedDocument {
    pub id: Uuid,
    pub business_id: Uuid,
    pub user_id: Uuid,
    pub document_type: String,
    pub document_name: Option<String>,
    pub file_data: Option<Vec<u8>>,
    pub file_format: Option<String>,
    pub file_size: Option<i64>,
    pub version: i32,
    pub generation_params: Option<serde_json::Value>,
    pub template_used: Option<String>,
    pub ai_model: Option<String>,
    pub token_usage: Option<i32>,
    pub status: String,
    pub download_count: i32,
    pub generated_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedDocumentResponse {
    pub id: Uuid,
    pub business_id: Uuid,
    pub document_type: String,
    pub document_name: String,
    pub file_format: String,
    pub file_size: i64,
    pub version: i32,
    pub template_used: Option<String>,
    pub status: String,
    pub download_count: i32,
    pub download_url: Option<String>,
    pub generated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

// =============================================================================
// Document Generation Requests
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateBusinessPlanRequest {
    pub business_id: Uuid,
    #[serde(default = "default_include_financials")]
    pub include_financials: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_sections: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub years_projection: Option<i32>, // 3 or 5 years
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratePitchDeckRequest {
    pub business_id: Uuid,
    #[serde(default = "default_template")]
    pub template: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slides_to_include: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateDocumentResponse {
    pub generation_id: Uuid,
    pub status: String,
    pub estimated_seconds: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStatusResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<GeneratedDocumentResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<i32>,
}

// =============================================================================
// Document Content Structures
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessPlanContent {
    pub executive_summary: ExecutiveSummary,
    pub company_overview: CompanyOverview,
    pub problem_statement: ProblemStatement,
    pub solution: SolutionSection,
    pub market_analysis: MarketAnalysis,
    pub competitive_analysis: CompetitiveAnalysis,
    pub business_model: BusinessModelSection,
    pub marketing_sales: MarketingSales,
    pub operations: OperationsSection,
    pub management_team: ManagementTeam,
    pub financial_projections: FinancialProjections,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funding_request: Option<FundingRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    pub mission_statement: String,
    pub problem_summary: String,
    pub solution_summary: String,
    pub market_opportunity: String,
    pub business_model_summary: String,
    pub team_overview: String,
    pub funding_needed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyOverview {
    pub company_name: String,
    pub legal_structure: String,
    pub location: String,
    pub history: String,
    pub vision: String,
    pub milestones: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemStatement {
    pub problem_description: String,
    pub who_experiences_it: String,
    pub current_solutions: String,
    pub gaps_in_current_solutions: String,
    pub cost_of_problem: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionSection {
    pub product_description: String,
    pub how_it_solves: String,
    pub unique_value_proposition: String,
    pub key_features: Vec<Feature>,
    pub benefits: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketAnalysis {
    pub tam: MarketSize, // Total Addressable Market
    pub sam: MarketSize, // Serviceable Addressable Market
    pub som: MarketSize, // Serviceable Obtainable Market
    pub market_trends: Vec<String>,
    pub target_segments: Vec<TargetSegment>,
    pub growth_projections: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSize {
    pub value: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetSegment {
    pub name: String,
    pub description: String,
    pub size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitiveAnalysis {
    pub direct_competitors: Vec<Competitor>,
    pub indirect_competitors: Vec<Competitor>,
    pub competitive_advantage: String,
    pub barriers_to_entry: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Competitor {
    pub name: String,
    pub description: String,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessModelSection {
    pub revenue_streams: Vec<RevenueStream>,
    pub pricing_strategy: String,
    pub unit_economics: Option<String>,
    pub sales_channels: Vec<String>,
    pub customer_acquisition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueStream {
    pub name: String,
    pub description: String,
    pub percentage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketingSales {
    pub go_to_market: String,
    pub marketing_channels: Vec<String>,
    pub sales_process: String,
    pub customer_lifecycle: String,
    pub partnerships: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationsSection {
    pub day_to_day: String,
    pub technology: String,
    pub supply_chain: Option<String>,
    pub key_partners: Vec<String>,
    pub regulatory_compliance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagementTeam {
    pub founders: Vec<TeamMember>,
    pub key_members: Vec<TeamMember>,
    pub advisors: Vec<TeamMember>,
    pub hiring_plan: String,
    pub org_structure: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub name: String,
    pub role: String,
    pub bio: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialProjections {
    pub years: i32,
    pub revenue_projections: Vec<YearProjection>,
    pub expense_breakdown: Vec<ExpenseCategory>,
    pub profit_loss_summary: String,
    pub cash_flow_summary: String,
    pub key_assumptions: Vec<String>,
    pub break_even_analysis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearProjection {
    pub year: i32,
    pub revenue: String,
    pub expenses: String,
    pub profit: String,
    pub growth_rate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseCategory {
    pub category: String,
    pub percentage: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingRequest {
    pub amount_requested: String,
    pub use_of_funds: Vec<FundUse>,
    pub valuation: Option<String>,
    pub roi_projections: Option<String>,
    pub exit_strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundUse {
    pub category: String,
    pub percentage: String,
    pub amount: String,
    pub description: String,
}

// =============================================================================
// Pitch Deck Content
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum PitchDeckSlide {
    Title {
        company_name: String,
        tagline: String,
        founder_names: String,
        contact_info: String,
    },
    Problem {
        title: String,
        content: serde_json::Value,
    },
    Solution {
        title: String,
        content: serde_json::Value,
    },
    Product {
        title: String,
        content: serde_json::Value,
    },
    Market {
        title: String,
        content: serde_json::Value,
    },
    BusinessModel {
        title: String,
        content: serde_json::Value,
    },
    Competition {
        title: String,
        content: serde_json::Value,
    },
    Traction {
        title: String,
        content: serde_json::Value,
    },
    Team {
        title: String,
        members: Vec<serde_json::Value>,
    },
    Financials {
        title: String,
        content: serde_json::Value,
    },
    Funding {
        title: String,
        content: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PitchDeckContent {
    pub title_slide: TitleSlide,
    pub problem_slide: ProblemSlide,
    pub solution_slide: SolutionSlide,
    pub product_slide: ProductSlide,
    pub market_slide: MarketSlide,
    pub business_model_slide: BusinessModelSlide,
    pub competition_slide: CompetitionSlide,
    pub traction_slide: TractionSlide,
    pub team_slide: TeamSlide,
    pub financials_slide: FinancialsSlide,
    pub funding_slide: FundingSlide,
    pub template: String,
    pub slides: Vec<PitchDeckSlide>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleSlide {
    pub company_name: String,
    pub tagline: String,
    pub logo_url: Option<String>,
    pub founder_names: String,
    pub contact_info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemSlide {
    pub problem_statement: String,
    pub visual_description: String,
    pub market_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionSlide {
    pub solution_description: String,
    pub key_benefits: Vec<String>,
    pub visual_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductSlide {
    pub product_name: String,
    pub description: String,
    pub key_features: Vec<String>,
    pub demo_placeholder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSlide {
    pub tam: String,
    pub sam: String,
    pub som: String,
    pub growth_rate: String,
    pub target_customer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessModelSlide {
    pub how_we_make_money: String,
    pub pricing_tiers: Vec<PricingTier>,
    pub unit_economics: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTier {
    pub name: String,
    pub price: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionSlide {
    pub competitive_matrix: Vec<CompetitivePosition>,
    pub moat_description: String,
    pub ip_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitivePosition {
    pub competitor: String,
    pub our_advantage: String,
    pub their_strength: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TractionSlide {
    pub key_achievements: Vec<String>,
    pub metrics: Vec<Metric>,
    pub partnerships: Vec<String>,
    pub press: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: String,
    pub timeframe: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSlide {
    pub founders: Vec<TeamMember>,
    pub key_hires: Vec<TeamMember>,
    pub advisors: Vec<TeamMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialsSlide {
    pub revenue_projection_chart: String,
    pub year_3_revenue: String,
    pub key_metrics: Vec<Metric>,
    pub funding_to_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingSlide {
    pub amount_raising: String,
    pub use_of_funds: Vec<FundUse>,
    pub timeline: String,
    pub investor_benefits: Vec<String>,
}

// =============================================================================
// Data Rooms
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DataRoom {
    pub id: Uuid,
    pub business_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub shareable_link: Option<String>,
    pub password_hash: Option<String>,
    pub password_protected: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub access_count: i32,
    pub download_limit: Option<i32>,
    pub watermark_enabled: bool,
    pub watermark_text: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Fields for investor data rooms
    pub is_public: Option<bool>,
    pub view_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRoomResponse {
    pub id: Uuid,
    pub business_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub shareable_link: Option<String>,
    pub password_protected: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub access_count: i32,
    pub download_limit: Option<i32>,
    pub watermark_enabled: bool,
    pub is_active: bool,
    pub file_count: i64,
    pub created_at: DateTime<Utc>,
    // Fields for investor data room responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DataRoomSummary {
    pub id: Uuid,
    pub business_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub shareable_link: Option<String>,
    pub password_hash: Option<String>,
    pub password_protected: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub access_count: i32,
    pub download_limit: Option<i32>,
    pub watermark_enabled: bool,
    pub watermark_text: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Additional fields from view
    pub business_name: String,
    pub file_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DataRoomFile {
    pub id: Uuid,
    pub data_room_id: Uuid,
    pub folder: String,
    pub file_name: String,
    pub file_data: Option<Vec<u8>>,
    pub file_mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub version: i32,
    pub description: Option<String>,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRoomFileResponse {
    pub id: Uuid,
    pub folder: String,
    pub file_name: String,
    pub file_mime_type: String,
    pub file_size: i64,
    pub version: i32,
    pub description: Option<String>,
    pub download_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DataRoomAccessLog {
    pub id: Uuid,
    pub data_room_id: Uuid,
    pub ip_address: Option<std::net::IpAddr>,
    pub user_agent: Option<String>,
    pub email: Option<String>,
    pub action: String,
    pub file_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

// DataRoomDocument for investor data room junction table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DataRoomDocument {
    pub id: Uuid,
    pub data_room_id: Uuid,
    pub document_id: Uuid,
    pub folder_path: Option<String>,
    pub order_index: i32,
    pub added_at: DateTime<Utc>,
}

// DataRoomAccess for investor access management
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DataRoomAccess {
    pub id: Uuid,
    pub data_room_id: Uuid,
    pub investor_id: Option<Uuid>,
    pub email: Option<String>,
    pub access_type: String,
    pub granted_by: Uuid,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_accessed_at: Option<DateTime<Utc>>,
}

// =============================================================================
// Data Room Requests
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDataRoomRequest {
    pub business_id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareDataRoomRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in_days: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watermark_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareDataRoomResponse {
    pub share_link: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub password_protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDataRoomFileRequest {
    pub folder: String,
    pub file_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub file_data: String, // base64 encoded
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDocumentToDataRoomRequest {
    pub document_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessDataRoomRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRoomAccessResponse {
    pub data_room_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub files: Vec<DataRoomFileResponse>,
    pub watermark_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRoomAccessLogsResponse {
    pub access_logs: Vec<AccessLogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLogEntry {
    pub id: Uuid,
    pub ip_address: Option<String>,
    pub email: Option<String>,
    pub action: String,
    pub file_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Document Types Enum
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentType {
    BusinessPlan,
    PitchDeck,
    OnePager,
    BrandGuidelines,
    FinancialModel,
}

impl DocumentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BusinessPlan => "business_plan",
            Self::PitchDeck => "pitch_deck",
            Self::OnePager => "one_pager",
            Self::BrandGuidelines => "brand_guidelines",
            Self::FinancialModel => "financial_model",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::BusinessPlan => "Business Plan",
            Self::PitchDeck => "Pitch Deck",
            Self::OnePager => "One Pager",
            Self::BrandGuidelines => "Brand Guidelines",
            Self::FinancialModel => "Financial Model",
        }
    }

    pub fn default_format(&self) -> &'static str {
        match self {
            Self::BusinessPlan => "pdf",
            Self::PitchDeck => "pdf",
            Self::OnePager => "pdf",
            Self::BrandGuidelines => "pdf",
            Self::FinancialModel => "xlsx",
        }
    }
}

impl std::str::FromStr for DocumentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "business_plan" => Ok(Self::BusinessPlan),
            "pitch_deck" => Ok(Self::PitchDeck),
            "one_pager" => Ok(Self::OnePager),
            "brand_guidelines" => Ok(Self::BrandGuidelines),
            "financial_model" => Ok(Self::FinancialModel),
            _ => Err(format!("Unknown document type: {}", s)),
        }
    }
}

// =============================================================================
// Data Room Folder Types
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataRoomFolder {
    ExecutiveSummary,
    PitchDeck,
    BusinessPlan,
    Financials,
    Legal,
    Team,
    Product,
    MarketResearch,
    Other,
}

impl DataRoomFolder {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExecutiveSummary => "executive_summary",
            Self::PitchDeck => "pitch_deck",
            Self::BusinessPlan => "business_plan",
            Self::Financials => "financials",
            Self::Legal => "legal",
            Self::Team => "team",
            Self::Product => "product",
            Self::MarketResearch => "market_research",
            Self::Other => "other",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ExecutiveSummary => "01 Executive Summary",
            Self::PitchDeck => "02 Pitch Deck",
            Self::BusinessPlan => "03 Business Plan",
            Self::Financials => "04 Financials",
            Self::Legal => "05 Legal",
            Self::Team => "06 Team",
            Self::Product => "07 Product",
            Self::MarketResearch => "08 Market Research",
            Self::Other => "09 Other",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::ExecutiveSummary,
            Self::PitchDeck,
            Self::BusinessPlan,
            Self::Financials,
            Self::Legal,
            Self::Team,
            Self::Product,
            Self::MarketResearch,
            Self::Other,
        ]
    }
}

impl std::str::FromStr for DataRoomFolder {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "executive_summary" => Ok(Self::ExecutiveSummary),
            "pitch_deck" => Ok(Self::PitchDeck),
            "business_plan" => Ok(Self::BusinessPlan),
            "financials" => Ok(Self::Financials),
            "legal" => Ok(Self::Legal),
            "team" => Ok(Self::Team),
            "product" => Ok(Self::Product),
            "market_research" => Ok(Self::MarketResearch),
            "other" => Ok(Self::Other),
            _ => Err(format!("Unknown folder: {}", s)),
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn default_include_financials() -> bool {
    true
}

fn default_template() -> String {
    "modern".to_string()
}

// =============================================================================
// Document Templates
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PitchDeckTemplate {
    pub code: String,
    pub name: String,
    pub description: String,
    pub primary_color: String,
    pub secondary_color: String,
    pub font_heading: String,
    pub font_body: String,
    pub slide_background: String,
    pub accent_style: String,
}

pub fn get_pitch_deck_templates() -> Vec<PitchDeckTemplate> {
    vec![
        PitchDeckTemplate {
            code: "modern".to_string(),
            name: "Modern Clean".to_string(),
            description: "Clean, professional design with minimal elements".to_string(),
            primary_color: "#2563EB".to_string(),
            secondary_color: "#7C3AED".to_string(),
            font_heading: "Inter".to_string(),
            font_body: "Inter".to_string(),
            slide_background: "#FFFFFF".to_string(),
            accent_style: "gradient".to_string(),
        },
        PitchDeckTemplate {
            code: "bold".to_string(),
            name: "Bold Impact".to_string(),
            description: "High contrast, bold typography for maximum impact".to_string(),
            primary_color: "#111827".to_string(),
            secondary_color: "#DC2626".to_string(),
            font_heading: "Space Grotesk".to_string(),
            font_body: "Inter".to_string(),
            slide_background: "#FFFFFF".to_string(),
            accent_style: "solid".to_string(),
        },
        PitchDeckTemplate {
            code: "startup".to_string(),
            name: "Startup Vibes".to_string(),
            description: "Friendly, approachable design for early-stage startups".to_string(),
            primary_color: "#7C3AED".to_string(),
            secondary_color: "#EC4899".to_string(),
            font_heading: "Poppins".to_string(),
            font_body: "Open Sans".to_string(),
            slide_background: "#F9FAFB".to_string(),
            accent_style: "rounded".to_string(),
        },
        PitchDeckTemplate {
            code: "dark".to_string(),
            name: "Dark Mode".to_string(),
            description: "Sleek dark background with vibrant accents".to_string(),
            primary_color: "#3B82F6".to_string(),
            secondary_color: "#10B981".to_string(),
            font_heading: "Montserrat".to_string(),
            font_body: "Inter".to_string(),
            slide_background: "#111827".to_string(),
            accent_style: "glow".to_string(),
        },
    ]
}
