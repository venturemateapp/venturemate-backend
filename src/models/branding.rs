//! Branding Kit Models
//! 
//! Models for logo generation, color palettes, font pairings,
//! and complete brand asset management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// =============================================================================
// Brand Assets
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BrandAsset {
    pub id: Uuid,
    pub business_id: Uuid,
    pub logo_data: Option<Vec<u8>>,
    pub logo_mime_type: Option<String>,
    pub logo_variants: Option<serde_json::Value>,
    pub logo_generation_prompt: Option<String>,
    pub logo_generation_model: Option<String>,
    pub color_palette: Option<serde_json::Value>,
    pub font_pairings: Option<serde_json::Value>,
    pub brand_guidelines_pdf: Option<Vec<u8>>,
    pub status: String,
    pub generated_at: Option<DateTime<Utc>>,
    pub downloaded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandAssetResponse {
    pub id: Uuid,
    pub business_id: Uuid,
    pub logo_url: Option<String>,
    pub logo_variants: LogoVariants,
    pub color_palette: ColorPalette,
    pub font_pairings: FontPairings,
    pub brand_guidelines_available: bool,
    pub status: String,
    pub generated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogoVariants {
    pub full_color: Option<String>,
    pub icon: Option<String>,
    pub white: Option<String>,
    pub horizontal: Option<String>,
}

// =============================================================================
// Color Palette
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColorPalette {
    pub primary: ColorInfo,
    pub secondary: ColorInfo,
    pub accent: Option<ColorInfo>,
    pub neutral: NeutralColors,
    pub functional: FunctionalColors,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColorInfo {
    pub name: String,
    pub hex: String,
    pub rgb: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hsl: Option<String>,
    pub usage: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NeutralColors {
    pub dark: String,
    pub medium: String,
    pub light: String,
    pub white: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FunctionalColors {
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
}

// =============================================================================
// Font Pairings
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FontPairings {
    pub heading: FontInfo,
    pub body: FontInfo,
    pub fallback: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FontInfo {
    pub name: String,
    pub category: String,
    pub google_url: String,
    pub weights: Vec<i32>,
    pub usage: String,
}

// =============================================================================
// Brand Generation Request/Response
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateBrandKitRequest {
    pub business_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_personality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_prompt_additions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateBrandKitResponse {
    pub generation_id: Uuid,
    pub status: String,
    pub estimated_seconds: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandKitStatusResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<BrandAssetResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegenerateLogoRequest {
    pub business_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_preference: Option<String>, // 'modern', 'classic', 'minimal', 'bold'
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPaletteRequest {
    pub business_id: Uuid,
    pub industry: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_personality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_primary: Option<String>, // hex color
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontPairingRequest {
    pub business_id: Uuid,
    pub industry: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vibe: Option<String>, // 'professional', 'friendly', 'elegant', 'bold'
}

// =============================================================================
// Brand Assets Log
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BrandAssetsLog {
    pub id: Uuid,
    pub business_id: Uuid,
    pub generation_type: String,
    pub prompt_used: Option<String>,
    pub model_used: Option<String>,
    pub response_time_ms: Option<i32>,
    pub success: bool,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Color Palette Presets
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ColorPalettePreset {
    pub id: Uuid,
    pub name: String,
    pub category: Option<String>,
    pub primary_hex: String,
    pub secondary_hex: Option<String>,
    pub accent_hex: Option<String>,
    pub neutral_dark_hex: Option<String>,
    pub neutral_light_hex: Option<String>,
    pub success_hex: String,
    pub warning_hex: String,
    pub error_hex: String,
    pub info_hex: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Font Pairing Presets
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FontPairingPreset {
    pub id: Uuid,
    pub name: String,
    pub category: Option<String>,
    pub heading_font: String,
    pub heading_weights: Option<Vec<i32>>,
    pub heading_google_url: Option<String>,
    pub body_font: String,
    pub body_weights: Option<Vec<i32>>,
    pub body_google_url: Option<String>,
    pub fallback_font: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Brand Guidelines PDF Generation
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandGuidelinesData {
    pub business_name: String,
    pub tagline: Option<String>,
    pub logo_variants: LogoVariants,
    pub color_palette: ColorPalette,
    pub font_pairings: FontPairings,
    pub brand_voice: Option<String>,
    pub dos_and_donts: DosAndDonts,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DosAndDonts {
    pub dos: Vec<String>,
    pub donts: Vec<String>,
}

// =============================================================================
// Industry Color Psychology Mapping
// =============================================================================

#[derive(Debug, Clone, Copy)]
pub enum IndustryColorPsychology {
    Fintech,
    Agritech,
    Healthtech,
    Edtech,
    Ecommerce,
    Saas,
    Marketplace,
    Cleantech,
    Media,
    Proptech,
    Other,
}

impl IndustryColorPsychology {
    pub fn from_industry(industry: &str) -> Self {
        match industry.to_lowercase().as_str() {
            "fintech" | "financial services" => Self::Fintech,
            "agritech" | "agriculture" => Self::Agritech,
            "healthtech" | "healthcare" => Self::Healthtech,
            "edtech" | "education" => Self::Edtech,
            "ecommerce" | "retail" => Self::Ecommerce,
            "saas" | "software" => Self::Saas,
            "marketplace" => Self::Marketplace,
            "cleantech" | "energy" => Self::Cleantech,
            "media" | "content" => Self::Media,
            "proptech" | "real estate" => Self::Proptech,
            _ => Self::Other,
        }
    }

    pub fn get_primary_color(&self) -> &'static str {
        match self {
            Self::Fintech => "#2563EB",      // Trust Blue
            Self::Agritech => "#059669",     // Growth Green
            Self::Healthtech => "#0D9488",   // Healing Teal
            Self::Edtech => "#F59E0B",       // Learning Yellow
            Self::Ecommerce => "#EA580C",    // Energy Orange
            Self::Saas => "#4F46E5",         // Professional Indigo
            Self::Marketplace => "#7C3AED",  // Connection Purple
            Self::Cleantech => "#16A34A",    // Eco Green
            Self::Media => "#DC2626",        // Attention Red
            Self::Proptech => "#0891B2",     // Trust Cyan
            Self::Other => "#4F46E5",        // Default Indigo
        }
    }

    pub fn get_secondary_color(&self) -> &'static str {
        match self {
            Self::Fintech => "#7C3AED",      // Innovation Purple
            Self::Agritech => "#D97706",     // Earth Orange
            Self::Healthtech => "#3B82F6",   // Care Blue
            Self::Edtech => "#10B981",       // Growth Green
            Self::Ecommerce => "#2563EB",    // Trust Blue
            Self::Saas => "#06B6D4",         // Tech Cyan
            Self::Marketplace => "#EC4899",  // Energy Pink
            Self::Cleantech => "#0891B2",    // Water Blue
            Self::Media => "#F59E0B",        // Creative Orange
            Self::Proptech => "#7C3AED",     // Innovation Purple
            Self::Other => "#7C3AED",        // Default Purple
        }
    }

    pub fn get_color_meaning(&self) -> Vec<String> {
        match self {
            Self::Fintech => vec!["Trust".to_string(), "Innovation".to_string(), "Security".to_string()],
            Self::Agritech => vec!["Growth".to_string(), "Earth".to_string(), "Vitality".to_string()],
            Self::Healthtech => vec!["Healing".to_string(), "Trust".to_string(), "Care".to_string()],
            Self::Edtech => vec!["Learning".to_string(), "Growth".to_string(), "Innovation".to_string()],
            Self::Ecommerce => vec!["Energy".to_string(), "Trust".to_string(), "Excitement".to_string()],
            Self::Saas => vec!["Professional".to_string(), "Innovation".to_string(), "Clean".to_string()],
            Self::Marketplace => vec!["Connection".to_string(), "Trust".to_string(), "Energy".to_string()],
            Self::Cleantech => vec!["Eco".to_string(), "Growth".to_string(), "Future".to_string()],
            Self::Media => vec!["Attention".to_string(), "Creative".to_string(), "Passion".to_string()],
            Self::Proptech => vec!["Trust".to_string(), "Innovation".to_string(), "Stability".to_string()],
            Self::Other => vec!["Professional".to_string(), "Innovation".to_string(), "Trust".to_string()],
        }
    }
}

// =============================================================================
// Font Pairing by Industry
// =============================================================================

pub fn get_industry_font_pairing(industry: &str) -> FontPairings {
    match industry.to_lowercase().as_str() {
        "fintech" | "financial services" => FontPairings {
            heading: FontInfo {
                name: "Montserrat".to_string(),
                category: "sans-serif".to_string(),
                google_url: "https://fonts.googleapis.com/css2?family=Montserrat:wght@400;600;700&display=swap".to_string(),
                weights: vec![400, 600, 700],
                usage: "Headlines, titles, navigation".to_string(),
            },
            body: FontInfo {
                name: "Open Sans".to_string(),
                category: "sans-serif".to_string(),
                google_url: "https://fonts.googleapis.com/css2?family=Open+Sans:wght@400;600&display=swap".to_string(),
                weights: vec![400, 600],
                usage: "Body text, paragraphs".to_string(),
            },
            fallback: "system-ui, -apple-system, sans-serif".to_string(),
        },
        "agritech" | "agriculture" => FontPairings {
            heading: FontInfo {
                name: "Poppins".to_string(),
                category: "sans-serif".to_string(),
                google_url: "https://fonts.googleapis.com/css2?family=Poppins:wght@400;600;700&display=swap".to_string(),
                weights: vec![400, 600, 700],
                usage: "Headlines, titles".to_string(),
            },
            body: FontInfo {
                name: "Lato".to_string(),
                category: "sans-serif".to_string(),
                google_url: "https://fonts.googleapis.com/css2?family=Lato:wght@400;600&display=swap".to_string(),
                weights: vec![400, 600],
                usage: "Body text, paragraphs".to_string(),
            },
            fallback: "system-ui, -apple-system, sans-serif".to_string(),
        },
        "healthtech" | "healthcare" => FontPairings {
            heading: FontInfo {
                name: "Raleway".to_string(),
                category: "sans-serif".to_string(),
                google_url: "https://fonts.googleapis.com/css2?family=Raleway:wght@400;600;700&display=swap".to_string(),
                weights: vec![400, 600, 700],
                usage: "Headlines, titles".to_string(),
            },
            body: FontInfo {
                name: "Source Sans Pro".to_string(),
                category: "sans-serif".to_string(),
                google_url: "https://fonts.googleapis.com/css2?family=Source+Sans+Pro:wght@400;600&display=swap".to_string(),
                weights: vec![400, 600],
                usage: "Body text, paragraphs".to_string(),
            },
            fallback: "system-ui, -apple-system, sans-serif".to_string(),
        },
        "edtech" | "education" => FontPairings {
            heading: FontInfo {
                name: "Nunito".to_string(),
                category: "sans-serif".to_string(),
                google_url: "https://fonts.googleapis.com/css2?family=Nunito:wght@400;600;700&display=swap".to_string(),
                weights: vec![400, 600, 700],
                usage: "Headlines, titles".to_string(),
            },
            body: FontInfo {
                name: "Roboto".to_string(),
                category: "sans-serif".to_string(),
                google_url: "https://fonts.googleapis.com/css2?family=Roboto:wght@400;600&display=swap".to_string(),
                weights: vec![400, 600],
                usage: "Body text, paragraphs".to_string(),
            },
            fallback: "system-ui, -apple-system, sans-serif".to_string(),
        },
        "ecommerce" | "retail" => FontPairings {
            heading: FontInfo {
                name: "Playfair Display".to_string(),
                category: "serif".to_string(),
                google_url: "https://fonts.googleapis.com/css2?family=Playfair+Display:wght@400;600;700&display=swap".to_string(),
                weights: vec![400, 600, 700],
                usage: "Headlines, titles".to_string(),
            },
            body: FontInfo {
                name: "Inter".to_string(),
                category: "sans-serif".to_string(),
                google_url: "https://fonts.googleapis.com/css2?family=Inter:wght@400;600&display=swap".to_string(),
                weights: vec![400, 600],
                usage: "Body text, paragraphs".to_string(),
            },
            fallback: "Georgia, serif".to_string(),
        },
        "saas" | "software" => FontPairings {
            heading: FontInfo {
                name: "Inter".to_string(),
                category: "sans-serif".to_string(),
                google_url: "https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap".to_string(),
                weights: vec![400, 600, 700],
                usage: "Headlines, titles, navigation".to_string(),
            },
            body: FontInfo {
                name: "Inter".to_string(),
                category: "sans-serif".to_string(),
                google_url: "https://fonts.googleapis.com/css2?family=Inter:wght@400;600&display=swap".to_string(),
                weights: vec![400, 600],
                usage: "Body text, paragraphs".to_string(),
            },
            fallback: "system-ui, -apple-system, sans-serif".to_string(),
        },
        "marketplace" => FontPairings {
            heading: FontInfo {
                name: "Space Grotesk".to_string(),
                category: "sans-serif".to_string(),
                google_url: "https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;600;700&display=swap".to_string(),
                weights: vec![400, 600, 700],
                usage: "Headlines, titles".to_string(),
            },
            body: FontInfo {
                name: "Work Sans".to_string(),
                category: "sans-serif".to_string(),
                google_url: "https://fonts.googleapis.com/css2?family=Work+Sans:wght@400;600&display=swap".to_string(),
                weights: vec![400, 600],
                usage: "Body text, paragraphs".to_string(),
            },
            fallback: "system-ui, -apple-system, sans-serif".to_string(),
        },
        _ => FontPairings {
            heading: FontInfo {
                name: "Inter".to_string(),
                category: "sans-serif".to_string(),
                google_url: "https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap".to_string(),
                weights: vec![400, 600, 700],
                usage: "Headlines, titles".to_string(),
            },
            body: FontInfo {
                name: "Inter".to_string(),
                category: "sans-serif".to_string(),
                google_url: "https://fonts.googleapis.com/css2?family=Inter:wght@400;600&display=swap".to_string(),
                weights: vec![400, 600],
                usage: "Body text, paragraphs".to_string(),
            },
            fallback: "system-ui, -apple-system, sans-serif".to_string(),
        },
    }
}
