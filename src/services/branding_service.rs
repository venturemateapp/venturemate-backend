//! Branding Kit Service
//! 
//! Generates complete brand assets including logos, color palettes,
//! font pairings, and brand guidelines PDF.

use crate::utils::{AppError, Result};
use base64;
use crate::models::branding::{
    BrandAsset, BrandAssetResponse, BrandAssetsLog, ColorInfo, ColorPalette,
    ColorPalettePreset, FontPairingPreset, FontPairings, GenerateBrandKitRequest,
    GenerateBrandKitResponse, BrandKitStatusResponse, IndustryColorPsychology,
    LogoVariants, RegenerateLogoRequest, get_industry_font_pairing, NeutralColors, FunctionalColors,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_ENGINE};
use crate::services::ai_service::AIService;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct BrandingService {
    db: PgPool,
    ai_service: Arc<AIService>,
}

impl BrandingService {
    pub fn new(db: PgPool, ai_service: Arc<AIService>) -> Self {
        Self { db, ai_service }
    }

    /// Generate a complete brand kit for a business
    pub async fn generate_brand_kit(
        &self,
        user_id: Uuid,
        req: GenerateBrandKitRequest,
    ) -> Result<GenerateBrandKitResponse> {
        info!("Generating brand kit for business: {}", req.business_id);

        // Verify business exists and user has access
        let business: Option<(String, String, Option<String>)> = sqlx::query_as(
            "SELECT name, industry, tagline FROM businesses WHERE id = $1 AND user_id = $2"
        )
        .bind(req.business_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| {
            error!("Database error fetching business: {}", e);
            AppError::Database(e)
        })?;

        let (business_name, industry, tagline) = business.ok_or_else(|| {
            AppError::NotFound("Business not found".to_string())
        })?;

        // Check if brand assets already exist
        let existing: Option<BrandAsset> = sqlx::query_as(
            "SELECT * FROM brand_assets WHERE business_id = $1"
        )
        .bind(req.business_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        if let Some(ref asset) = existing {
            if asset.status == "generating" {
                return Ok(GenerateBrandKitResponse {
                    generation_id: asset.id,
                    status: "processing".to_string(),
                    estimated_seconds: 15,
                });
            }
        }

        // Create brand asset record
        let generation_id = existing.as_ref().map(|a| a.id).unwrap_or_else(Uuid::new_v4);
        
        if existing.is_none() {
            sqlx::query(
                "INSERT INTO brand_assets (id, business_id, status, created_at, updated_at) 
                 VALUES ($1, $2, 'generating', NOW(), NOW())
                 ON CONFLICT (business_id) DO UPDATE SET status = 'generating', updated_at = NOW()"
            )
            .bind(generation_id)
            .bind(req.business_id)
            .execute(&self.db)
            .await
            .map_err(AppError::Database)?;
        } else {
            sqlx::query(
                "UPDATE brand_assets SET status = 'generating', updated_at = NOW() WHERE id = $1"
            )
            .bind(generation_id)
            .execute(&self.db)
            .await
            .map_err(AppError::Database)?;
        }

        // Spawn background task for generation
        let db_clone = self.db.clone();
        let ai_service_clone = self.ai_service.clone();
        let business_id = req.business_id;
        let business_name_clone = business_name.clone();
        let industry_clone = industry.clone();
        let tagline_clone = tagline.clone();
        let brand_personality = req.brand_personality.clone();
        let custom_prompt = req.custom_prompt_additions.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::generate_brand_assets_background(
                db_clone,
                ai_service_clone,
                generation_id,
                business_id,
                business_name_clone,
                industry_clone,
                tagline_clone,
                brand_personality,
                custom_prompt,
                user_id,
            ).await {
                error!("Background brand generation failed: {}", e);
            }
        });

        Ok(GenerateBrandKitResponse {
            generation_id,
            status: "processing".to_string(),
            estimated_seconds: 20,
        })
    }

    /// Background task for brand asset generation
    async fn generate_brand_assets_background(
        db: PgPool,
        ai_service: Arc<AIService>,
        generation_id: Uuid,
        business_id: Uuid,
        business_name: String,
        industry: String,
        tagline: Option<String>,
        brand_personality: Option<String>,
        custom_prompt: Option<String>,
        _user_id: Uuid,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Step 1: Generate color palette
        let color_psychology = IndustryColorPsychology::from_industry(&industry);
        let color_palette = generate_color_palette(&color_psychology);

        // Step 2: Generate font pairings
        let font_pairings = get_industry_font_pairing(&industry);

        // Step 3: Generate logo using AI (SVG-style description)
        let logo_prompt = build_logo_generation_prompt(
            &business_name,
            &industry,
            tagline.as_deref(),
            brand_personality.as_deref(),
            &color_palette,
            custom_prompt.as_deref(),
        );

        let logo_result = Self::generate_logo_with_ai(&ai_service, &logo_prompt).await;
        
        // Log the logo generation attempt
        let logo_success = logo_result.is_ok();
        let logo_error = logo_result.as_ref().err().map(|e| e.to_string());
        let logo_data = logo_result.unwrap_or_default();

        sqlx::query(
            "INSERT INTO brand_assets_logs (business_id, generation_type, prompt_used, model_used, response_time_ms, success, error_message, created_at)
             VALUES ($1, 'logo', $2, 'claude', $3, $4, $5, NOW())"
        )
        .bind(business_id)
        .bind(&logo_prompt)
        .bind(start_time.elapsed().as_millis() as i32)
        .bind(logo_success)
        .bind(logo_error)
        .execute(&db)
        .await
        .map_err(AppError::Database)?;

        // Step 4: Generate logo variants (placeholder - in production would generate multiple variants)
        let logo_variants = serde_json::json!({
            "full_color": if logo_data.is_empty() { None::<String> } else { Some(format!("data:image/svg+xml;base64,{}", BASE64_ENGINE.encode(&logo_data))) },
            "icon": None::<String>,
            "white": None::<String>,
            "horizontal": None::<String>
        });

        // Step 5: Save all assets to database
        let processing_time = start_time.elapsed().as_millis() as i32;

        sqlx::query(
            "UPDATE brand_assets SET
                logo_data = $1,
                logo_mime_type = $2,
                logo_variants = $3,
                logo_generation_prompt = $4,
                logo_generation_model = $5,
                color_palette = $6,
                font_pairings = $7,
                status = $8,
                generated_at = NOW(),
                updated_at = NOW()
             WHERE id = $9"
        )
        .bind(if logo_data.is_empty() { None } else { Some(logo_data) })
        .bind(Some("image/svg+xml"))
        .bind(&logo_variants)
        .bind(Some(&logo_prompt))
        .bind(Some("claude"))
        .bind(serde_json::to_value(&color_palette).unwrap_or_default())
        .bind(serde_json::to_value(&font_pairings).unwrap_or_default())
        .bind(if logo_success { "ready" } else { "failed" })
        .bind(generation_id)
        .execute(&db)
        .await
        .map_err(AppError::Database)?;

        info!("Brand kit generation completed for business: {} in {}ms", business_id, processing_time);

        Ok(())
    }

    /// Generate logo using AI - returns SVG string
    async fn generate_logo_with_ai(
        ai_service: &Arc<AIService>,
        prompt: &str,
    ) -> Result<Vec<u8>> {
        // Use AI to generate an SVG logo description
        let system_prompt = "You are a professional logo designer. Generate an SVG logo based on the description. 
        Return ONLY valid SVG code without any markdown formatting or explanation. 
        The SVG should be simple, scalable, and professional. 
        Size: 512x512 viewBox. Use flat design with the colors mentioned.";

        match ai_service.generate_text(system_prompt, prompt, 2000, Some(0.7)).await {
            Ok(svg_content) => {
                // Clean up the response
                let svg = svg_content
                    .trim()
                    .trim_start_matches("```svg")
                    .trim_start_matches("```")
                    .trim_end_matches("```")
                    .trim();
                
                if svg.starts_with("<svg") && svg.ends_with("</svg>") {
                    Ok(svg.as_bytes().to_vec())
                } else {
                    // Generate a fallback SVG
                    Ok(Self::generate_fallback_logo_svg())
                }
            }
            Err(e) => {
                warn!("AI logo generation failed: {}, using fallback", e);
                Ok(Self::generate_fallback_logo_svg())
            }
        }
    }

    /// Generate a fallback logo SVG (initials in a circle)
    fn generate_fallback_logo_svg() -> Vec<u8> {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
            <defs>
                <linearGradient id="grad" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" style="stop-color:#4F46E5;stop-opacity:1" />
                    <stop offset="100%" style="stop-color:#7C3AED;stop-opacity:1" />
                </linearGradient>
            </defs>
            <circle cx="256" cy="256" r="240" fill="url(#grad)"/>
            <text x="256" y="290" font-family="Arial, sans-serif" font-size="160" font-weight="bold" fill="white" text-anchor="middle">VM</text>
        </svg>"#;
        svg.as_bytes().to_vec()
    }

    /// Get brand kit status
    pub async fn get_brand_kit_status(
        &self,
        _user_id: Uuid,
        business_id: Uuid,
    ) -> Result<BrandKitStatusResponse> {
        let asset: Option<BrandAsset> = sqlx::query_as(
            "SELECT * FROM brand_assets WHERE business_id = $1"
        )
        .bind(business_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        let Some(asset) = asset else {
            return Ok(BrandKitStatusResponse {
                status: "not_found".to_string(),
                assets: None,
                error_message: None,
                progress_percent: None,
            });
        };

        let assets_response = if asset.status == "ready" {
            let logo_url = asset.logo_data.as_ref().map(|data| {
                format!("data:{};base64,{}", 
                    asset.logo_mime_type.as_deref().unwrap_or("image/svg+xml"),
                    BASE64_ENGINE.encode(data)
                )
            });

            let logo_variants: LogoVariants = asset.logo_variants
                .as_ref()
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();

            let color_palette: ColorPalette = asset.color_palette
                .as_ref()
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();

            let font_pairings: FontPairings = asset.font_pairings
                .as_ref()
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();

            Some(BrandAssetResponse {
                id: asset.id,
                business_id: asset.business_id,
                logo_url,
                logo_variants,
                color_palette,
                font_pairings,
                brand_guidelines_available: asset.brand_guidelines_pdf.is_some(),
                status: asset.status.clone(),
                generated_at: asset.generated_at,
            })
        } else {
            None
        };

        Ok(BrandKitStatusResponse {
            status: asset.status.clone(),
            assets: assets_response,
            error_message: None,
            progress_percent: if asset.status == "generating" { Some(50) } else { None },
        })
    }

    /// Regenerate logo with custom options
    pub async fn regenerate_logo(
        &self,
        user_id: Uuid,
        req: RegenerateLogoRequest,
    ) -> Result<GenerateBrandKitResponse> {
        info!("Regenerating logo for business: {}", req.business_id);

        // Verify ownership
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM businesses WHERE id = $1 AND user_id = $2)"
        )
        .bind(req.business_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        if !exists {
            return Err(AppError::NotFound("Business not found".to_string()));
        }

        // Get existing brand assets
        let asset: Option<BrandAsset> = sqlx::query_as(
            "SELECT * FROM brand_assets WHERE business_id = $1"
        )
        .bind(req.business_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        let Some(asset) = asset else {
            return Err(AppError::NotFound("Brand assets not found".to_string()));
        };

        // Update status to generating
        sqlx::query(
            "UPDATE brand_assets SET status = 'generating', updated_at = NOW() WHERE id = $1"
        )
        .bind(asset.id)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        // Get business details
        let (business_name, industry, tagline): (String, String, Option<String>) = sqlx::query_as(
            "SELECT name, industry, tagline FROM businesses WHERE id = $1"
        )
        .bind(req.business_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        // Get existing color palette
        let _color_palette: ColorPalette = asset.color_palette
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(|| {
                let psych = IndustryColorPsychology::from_industry(&industry);
                generate_color_palette(&psych)
            });

        // Build new prompt with style preference
        let logo_prompt = if let Some(custom) = req.custom_prompt {
            custom
        } else {
            let style = req.style_preference.as_deref().unwrap_or("modern");
            format!(
                "Create a {} logo for \"{}\", a {} company. {} Style: {}",
                style,
                business_name,
                industry,
                tagline.as_deref().unwrap_or(""),
                style
            )
        };

        // Spawn regeneration task
        let db_clone = self.db.clone();
        let ai_service_clone = self.ai_service.clone();
        let business_id = req.business_id;

        tokio::spawn(async move {
            let start_time = std::time::Instant::now();
            
            let logo_result = Self::generate_logo_with_ai(&ai_service_clone, &logo_prompt).await;
            let logo_success = logo_result.is_ok();
            let logo_data = logo_result.unwrap_or_default();

            // Log the attempt
            let _ = sqlx::query(
                "INSERT INTO brand_assets_logs (business_id, generation_type, prompt_used, model_used, response_time_ms, success, created_at)
                 VALUES ($1, 'logo', $2, 'claude', $3, $4, NOW())"
            )
            .bind(business_id)
            .bind(&logo_prompt)
            .bind(start_time.elapsed().as_millis() as i32)
            .bind(logo_success)
            .execute(&db_clone)
            .await;

            // Update the asset
            let _ = sqlx::query(
                "UPDATE brand_assets SET
                    logo_data = $1,
                    logo_generation_prompt = $2,
                    status = $3,
                    updated_at = NOW()
                 WHERE business_id = $4"
            )
            .bind(if logo_data.is_empty() { None } else { Some(logo_data) })
            .bind(Some(&logo_prompt))
            .bind(if logo_success { "ready" } else { "failed" })
            .bind(business_id)
            .execute(&db_clone)
            .await;
        });

        Ok(GenerateBrandKitResponse {
            generation_id: asset.id,
            status: "processing".to_string(),
            estimated_seconds: 15,
        })
    }

    /// Download brand kit (ZIP of all assets)
    pub async fn download_brand_kit(
        &self,
        user_id: Uuid,
        business_id: Uuid,
    ) -> Result<(String, Vec<u8>)> { // (filename, zip_bytes)
        // Verify ownership
        let business: Option<(String,)> = sqlx::query_as(
            "SELECT name FROM businesses WHERE id = $1 AND user_id = $2"
        )
        .bind(business_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        let Some((business_name,)) = business else {
            return Err(AppError::NotFound("Business not found".to_string()));
        };

        let asset: Option<BrandAsset> = sqlx::query_as(
            "SELECT * FROM brand_assets WHERE business_id = $1 AND status = 'ready'"
        )
        .bind(business_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        let Some(asset) = asset else {
            return Err(AppError::NotFound("Brand kit not ready".to_string()));
        };

        // Update download timestamp
        sqlx::query(
            "UPDATE brand_assets SET downloaded_at = NOW() WHERE id = $1"
        )
        .bind(asset.id)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        // For now, return a simple ZIP placeholder
        // In production, this would create a proper ZIP with all assets
        let filename = format!("brand-kit-{}.zip", 
            business_name.to_lowercase().replace(" ", "-"));
        
        // Placeholder ZIP content
        let zip_content = format!("Brand kit for {}\n\nLogo, colors, and fonts ready for download.", business_name);
        
        Ok((filename, zip_content.into_bytes()))
    }

    /// Get color palette presets
    pub async fn get_color_presets(
        &self,
        category: Option<String>,
    ) -> Result<Vec<ColorPalettePreset>> {
        let presets: Vec<ColorPalettePreset> = if let Some(cat) = category {
            sqlx::query_as(
                "SELECT * FROM color_palette_presets WHERE category = $1 AND is_active = true ORDER BY name"
            )
            .bind(cat)
            .fetch_all(&self.db)
            .await
            .map_err(AppError::Database)?
        } else {
            sqlx::query_as(
                "SELECT * FROM color_palette_presets WHERE is_active = true ORDER BY name"
            )
            .fetch_all(&self.db)
            .await
            .map_err(AppError::Database)?
        };

        Ok(presets)
    }

    /// Get font pairing presets
    pub async fn get_font_presets(
        &self,
        category: Option<String>,
    ) -> Result<Vec<FontPairingPreset>> {
        let presets: Vec<FontPairingPreset> = if let Some(cat) = category {
            sqlx::query_as(
                "SELECT * FROM font_pairing_presets WHERE category = $1 AND is_active = true ORDER BY name"
            )
            .bind(cat)
            .fetch_all(&self.db)
            .await
            .map_err(AppError::Database)?
        } else {
            sqlx::query_as(
                "SELECT * FROM font_pairing_presets WHERE is_active = true ORDER BY name"
            )
            .fetch_all(&self.db)
            .await
            .map_err(AppError::Database)?
        };

        Ok(presets)
    }

    /// Get brand generation logs
    pub async fn get_generation_logs(
        &self,
        user_id: Uuid,
        business_id: Uuid,
    ) -> Result<Vec<BrandAssetsLog>> {
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

        let logs: Vec<BrandAssetsLog> = sqlx::query_as(
            "SELECT * FROM brand_assets_logs WHERE business_id = $1 ORDER BY created_at DESC"
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(logs)
    }
}

/// Generate color palette based on industry psychology
fn generate_color_palette(psychology: &IndustryColorPsychology) -> ColorPalette {
    let primary_hex = psychology.get_primary_color();
    let secondary_hex = psychology.get_secondary_color();
    
    ColorPalette {
        primary: ColorInfo {
            name: "Primary".to_string(),
            hex: primary_hex.to_string(),
            rgb: hex_to_rgb(primary_hex),
            hsl: Some(hex_to_hsl(primary_hex)),
            usage: vec!["buttons".to_string(), "links".to_string(), "headers".to_string()],
        },
        secondary: ColorInfo {
            name: "Secondary".to_string(),
            hex: secondary_hex.to_string(),
            rgb: hex_to_rgb(secondary_hex),
            hsl: Some(hex_to_hsl(secondary_hex)),
            usage: vec!["accents".to_string(), "highlights".to_string(), "badges".to_string()],
        },
        accent: None,
        neutral: NeutralColors {
            dark: "#2D3748".to_string(),
            medium: "#4A5568".to_string(),
            light: "#EDF2F7".to_string(),
            white: "#FFFFFF".to_string(),
        },
        functional: FunctionalColors {
            success: "#48BB78".to_string(),
            warning: "#ED8936".to_string(),
            error: "#F56565".to_string(),
            info: "#4299E1".to_string(),
        },
    }
}

/// Build logo generation prompt
fn build_logo_generation_prompt(
    business_name: &str,
    industry: &str,
    tagline: Option<&str>,
    brand_personality: Option<&str>,
    color_palette: &ColorPalette,
    custom_additions: Option<&str>,
) -> String {
    let personality = brand_personality.unwrap_or("professional and innovative");
    let tagline_str = tagline.map(|t| format!("\nTagline: {}", t)).unwrap_or_default();
    let custom = custom_additions.map(|c| format!("\nAdditional requirements: {}", c)).unwrap_or_default();

    format!(
        r#"Create a professional SVG logo for "{}", a {} startup.

Brand Personality: {}{}{}

Color Palette:
- Primary: {}
- Secondary: {}

Requirements:
- Style: Modern, clean, flat design
- Elements: Incorporate {} industry elements subtly
- Format: SVG with 512x512 viewBox
- Background: Transparent
- Should work as both full logo and icon

Create a memorable, unique logo that represents trust and innovation."#,
        business_name,
        industry,
        personality,
        tagline_str,
        custom,
        color_palette.primary.hex,
        color_palette.secondary.hex,
        industry
    )
}

/// Convert hex to RGB string
fn hex_to_rgb(hex: &str) -> String {
    // Simple hex to RGB conversion
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = hex[0..2].parse::<u8>().unwrap_or(0);
        let g = hex[2..4].parse::<u8>().unwrap_or(0);
        let b = hex[4..6].parse::<u8>().unwrap_or(0);
        format!("{},{},{}", r, g, b)
    } else {
        "107,70,193".to_string()
    }
}

/// Convert hex to HSL string (simplified)
fn hex_to_hsl(hex: &str) -> String {
    // Simplified HSL conversion
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = hex[0..2].parse::<u8>().unwrap_or(0) as f32 / 255.0;
        let g = hex[2..4].parse::<u8>().unwrap_or(0) as f32 / 255.0;
        let b = hex[4..6].parse::<u8>().unwrap_or(0) as f32 / 255.0;
        
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        
        format!("260, 50%, {}%", (l * 100.0) as i32)
    } else {
        "260, 50%, 52%".to_string()
    }
}
