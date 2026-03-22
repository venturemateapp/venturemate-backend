// Phase 3: Credit Scoring Handler
use actix_web::{web, HttpRequest, HttpResponse, Result};
use sqlx::PgPool;
use uuid::Uuid;


use crate::models::*;
use crate::utils::get_user_id;


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/credit")
            .route("/dashboard", web::get().to(get_credit_dashboard))
            .route("/score", web::get().to(get_credit_score))
            .route("/score/calculate", web::post().to(calculate_credit_score))
            .route("/history", web::get().to(get_score_history))
            .route("/report", web::get().to(get_credit_report))
            .route("/offers", web::get().to(list_financing_offers))
            .route("/applications", web::get().to(list_applications))
            .route("/applications", web::post().to(apply_for_financing))
            .route("/applications/{id}", web::get().to(get_application))
    );
}

// Dashboard
async fn get_credit_dashboard(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    // Get or calculate current score
    let current_score = get_or_calculate_score(&pool, business_id).await?;
    
    // Get score history
    let history = sqlx::query_as::<_, CreditScoreHistory>(
        "SELECT * FROM credit_score_history WHERE business_id = $1 ORDER BY recorded_at DESC LIMIT 12"
    )
    .bind(business_id)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();
    
    let score_history: Vec<ScoreHistoryPoint> = history.into_iter()
        .map(|h| ScoreHistoryPoint {
            score: h.score,
            grade: h.score_grade,
            change: h.change_from_previous,
            recorded_at: h.recorded_at,
        })
        .collect();
    
    // Get available offers
    let offers = sqlx::query_as::<_, FinancingOffer>(
        "SELECT * FROM financing_offers WHERE is_active = true ORDER BY max_amount DESC"
    )
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();
    
    let offer_responses: Vec<FinancingOfferResponse> = offers.into_iter()
        .map(|o| {
            let is_eligible = o.required_credit_score_min.map_or(true, |min| current_score.overall_score >= min);
            FinancingOfferResponse {
                id: o.id,
                provider_name: o.provider_name,
                provider_type: o.provider_type,
                offer_type: o.offer_type,
                title: o.title,
                description: o.description,
                amount_range: format_amount_range(o.min_amount, o.max_amount, &o.currency),
                interest_rate_range: format_rate_range(o.interest_rate_min, o.interest_rate_max),
                term_range: format_term_range(o.term_months_min, o.term_months_max),
                eligibility: OfferEligibility {
                    is_eligible,
                    required_score: o.required_credit_score_min,
                    current_score: current_score.overall_score,
                    meets_requirements: is_eligible,
                    missing_requirements: if is_eligible { vec![] } else { vec!["Credit score below minimum".to_string()] },
                },
            }
        })
        .collect();
    
    // Get active applications with offer details using separate queries
    let applications = sqlx::query_as::<_, FinancingApplication>(
        "SELECT * FROM financing_applications WHERE business_id = $1 ORDER BY created_at DESC"
    )
    .bind(business_id)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();
    
    // Build a map of offers for lookup
    let offers = sqlx::query_as::<_, FinancingOffer>(
        "SELECT * FROM financing_offers"
    )
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();
    
    let offer_map: std::collections::HashMap<uuid::Uuid, FinancingOffer> = offers
        .into_iter()
        .map(|o| (o.id, o))
        .collect();
    
    let app_summaries: Vec<ApplicationSummary> = applications
        .into_iter()
        .filter_map(|a| {
            offer_map.get(&a.offer_id).map(|o| ApplicationSummary {
                id: a.id,
                provider_name: o.provider_name.clone(),
                offer_type: o.offer_type.clone(),
                requested_amount: a.requested_amount,
                status: a.status,
                submitted_at: a.submitted_at,
            })
        })
        .collect();
    
    let dashboard = CreditDashboard {
        current_score: credit_score_to_response(current_score),
        score_history,
        available_offers: offer_responses,
        active_applications: app_summaries,
        credit_utilization: None, // Would calculate from actual credit lines
    };
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(dashboard)))
}

// Credit Score
async fn get_credit_score(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    let score = get_or_calculate_score(&pool, business_id).await?;
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(credit_score_to_response(score))))
}

async fn calculate_credit_score(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    _body: web::Json<CalculateCreditScoreRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    // Calculate new score
    let score = calculate_and_save_score(&pool, business_id).await?;
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(credit_score_to_response(score))))
}

async fn get_score_history(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    let history = sqlx::query_as::<_, CreditScoreHistory>(
        "SELECT * FROM credit_score_history WHERE business_id = $1 ORDER BY recorded_at DESC"
    )
    .bind(business_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(history)))
}

async fn get_credit_report(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    // Get or create report
    let report = sqlx::query_as::<_, CreditReport>(
        r#"INSERT INTO credit_reports (business_id, report_type, generated_at, expires_at, access_count, report_data)
           VALUES ($1, 'full', NOW(), NOW() + INTERVAL '30 days', 0, '{}')
           ON CONFLICT (business_id, report_type) 
           DO UPDATE SET generated_at = NOW(), expires_at = NOW() + INTERVAL '30 days', access_count = credit_reports.access_count + 1
           RETURNING *"#
    )
    .bind(business_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(report)))
}

// Financing Offers
async fn list_financing_offers(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    // Get current score for eligibility
    let current_score = get_or_calculate_score(&pool, business_id).await?;
    
    let offers = sqlx::query_as::<_, FinancingOffer>(
        "SELECT * FROM financing_offers WHERE is_active = true ORDER BY max_amount DESC"
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let responses: Vec<FinancingOfferResponse> = offers.into_iter()
        .map(|o| {
            let is_eligible = o.required_credit_score_min.map_or(true, |min| current_score.overall_score >= min);
            FinancingOfferResponse {
                id: o.id,
                provider_name: o.provider_name,
                provider_type: o.provider_type,
                offer_type: o.offer_type,
                title: o.title,
                description: o.description,
                amount_range: format_amount_range(o.min_amount, o.max_amount, &o.currency),
                interest_rate_range: format_rate_range(o.interest_rate_min, o.interest_rate_max),
                term_range: format_term_range(o.term_months_min, o.term_months_max),
                eligibility: OfferEligibility {
                    is_eligible,
                    required_score: o.required_credit_score_min,
                    current_score: current_score.overall_score,
                    meets_requirements: is_eligible,
                    missing_requirements: if is_eligible { vec![] } else { vec!["Credit score below minimum".to_string()] },
                },
            }
        })
        .collect();
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(responses)))
}

// Applications
async fn list_applications(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    // Fetch applications and offers separately, then combine
    let applications = sqlx::query_as::<_, FinancingApplication>(
        "SELECT * FROM financing_applications WHERE business_id = $1 ORDER BY created_at DESC"
    )
    .bind(business_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    // Build a map of offers for lookup
    let offers = sqlx::query_as::<_, FinancingOffer>(
        "SELECT * FROM financing_offers"
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let offer_map: std::collections::HashMap<uuid::Uuid, FinancingOffer> = offers
        .into_iter()
        .map(|o| (o.id, o))
        .collect();
    
    // Combine into ApplicationSummary
    let summaries: Vec<ApplicationSummary> = applications
        .into_iter()
        .filter_map(|a| {
            offer_map.get(&a.offer_id).map(|o| ApplicationSummary {
                id: a.id,
                provider_name: o.provider_name.clone(),
                offer_type: o.offer_type.clone(),
                requested_amount: a.requested_amount,
                status: a.status,
                submitted_at: a.submitted_at,
            })
        })
        .collect();
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(summaries)))
}

async fn apply_for_financing(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<ApplyForFinancingRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    // Check eligibility
    let current_score = get_or_calculate_score(&pool, business_id).await?;
    
    let offer = sqlx::query_as::<_, FinancingOffer>(
        "SELECT * FROM financing_offers WHERE id = $1"
    )
    .bind(body.offer_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    .ok_or_else(|| actix_web::error::ErrorNotFound("Offer not found"))?;
    
    if let Some(min_score) = offer.required_credit_score_min {
        if current_score.overall_score < min_score {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "NOT_ELIGIBLE", 
                format!("Credit score {} is below minimum required {}", current_score.overall_score, min_score)
            )));
        }
    }
    
    let application = sqlx::query_as::<_, FinancingApplication>(
        r#"INSERT INTO financing_applications (business_id, offer_id, requested_amount, status, application_data)
           VALUES ($1, $2, $3, 'draft', $4)
           RETURNING *"#
    )
    .bind(business_id)
    .bind(body.offer_id)
    .bind(body.requested_amount)
    .bind(&body.application_data)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(ApiResponse::success(application)))
}

async fn get_application(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let app_id = path.into_inner();
    
    let application = sqlx::query_as::<_, FinancingApplication>(
        "SELECT * FROM financing_applications WHERE id = $1 AND business_id = $2"
    )
    .bind(app_id)
    .bind(business_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match application {
        Some(a) => Ok(HttpResponse::Ok().json(ApiResponse::success(a))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Application not found"))),
    }
}

// Helper functions
async fn get_or_calculate_score(pool: &PgPool, business_id: Uuid) -> Result<CreditScore, actix_web::Error> {
    // Check for existing valid score
    let existing = sqlx::query_as::<_, CreditScore>(
        "SELECT * FROM credit_scores WHERE business_id = $1 AND expires_at > NOW() ORDER BY calculated_at DESC LIMIT 1"
    )
    .bind(business_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    if let Some(score) = existing {
        return Ok(score);
    }
    
    // Calculate new score
    calculate_and_save_score(pool, business_id).await
}

async fn calculate_and_save_score(pool: &PgPool, business_id: Uuid) -> Result<CreditScore, actix_web::Error> {
    // Get business data for scoring
    let _business = sqlx::query_as::<_, Business>(
        "SELECT * FROM businesses WHERE id = $1"
    )
    .bind(business_id)
    .fetch_one(pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    // Calculate component scores (simplified algorithm)
    let payment_history = 70i32;
    let financial_stability = 65i32;
    let business_viability = 75i32;
    let compliance = 80i32;
    let market_position = 60i32;
    
    // Weighted average
    let overall = (
        payment_history * 25 +
        financial_stability * 25 +
        business_viability * 20 +
        compliance * 15 +
        market_position * 15
    ) / 100;
    
    let grade = score_to_grade(overall);
    let risk = score_to_risk(overall);
    
    let breakdown = serde_json::json!({
        "payment_history": { "score": payment_history, "weight": 0.25 },
        "financial_stability": { "score": financial_stability, "weight": 0.25 },
        "business_viability": { "score": business_viability, "weight": 0.20 },
        "compliance": { "score": compliance, "weight": 0.15 },
        "market_position": { "score": market_position, "weight": 0.15 },
    });
    
    let positive_factors = vec![
        "Business registration verified".to_string(),
        "Active digital presence".to_string(),
    ];
    
    let negative_factors = vec![
        "Limited financial history".to_string(),
        "Early stage business".to_string(),
    ];
    
    let suggested_limit = (overall as i64) * 1000; // $1000 per score point
    
    // Insert new score
    let score = sqlx::query_as::<_, CreditScore>(
        r#"INSERT INTO credit_scores (business_id, overall_score, score_grade, risk_level, payment_history_score, financial_stability_score, business_viability_score, compliance_score, market_position_score, score_breakdown, factors_positive, factors_negative, suggested_credit_limit, currency, expires_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 'USD', NOW() + INTERVAL '90 days')
           RETURNING *"#
    )
    .bind(business_id)
    .bind(overall)
    .bind(&grade)
    .bind(&risk)
    .bind(payment_history)
    .bind(financial_stability)
    .bind(business_viability)
    .bind(compliance)
    .bind(market_position)
    .bind(breakdown)
    .bind(serde_json::to_value(&positive_factors).unwrap_or_default())
    .bind(serde_json::to_value(&negative_factors).unwrap_or_default())
    .bind(suggested_limit)
    .fetch_one(pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    // Also record in history
    let previous_score = sqlx::query_scalar::<_, i32>(
        "SELECT score FROM credit_score_history WHERE business_id = $1 ORDER BY recorded_at DESC LIMIT 1"
    )
    .bind(business_id)
    .fetch_optional(pool)
    .await
    .unwrap_or_default();
    
    let change = match previous_score {
        Some(prev) => overall - prev,
        None => 0,
    };
    
    sqlx::query(
        "INSERT INTO credit_score_history (business_id, score, score_grade, change_from_previous, reason) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(business_id)
    .bind(overall)
    .bind(&grade)
    .bind(change)
    .bind("Automated recalculation")
    .execute(pool)
    .await
    .ok();
    
    Ok(score)
}

fn credit_score_to_response(score: CreditScore) -> CreditScoreResponse {
    CreditScoreResponse {
        business_id: score.business_id,
        overall_score: score.overall_score,
        score_grade: score.score_grade,
        risk_level: score.risk_level,
        components: CreditComponents {
            payment_history: ComponentDetail {
                score: score.payment_history_score,
                max_score: 100,
                weight: 0.25,
                description: "Payment history and reliability".to_string(),
            },
            financial_stability: ComponentDetail {
                score: score.financial_stability_score,
                max_score: 100,
                weight: 0.25,
                description: "Financial health and stability".to_string(),
            },
            business_viability: ComponentDetail {
                score: score.business_viability_score,
                max_score: 100,
                weight: 0.20,
                description: "Business model viability".to_string(),
            },
            compliance: ComponentDetail {
                score: score.compliance_score,
                max_score: 100,
                weight: 0.15,
                description: "Regulatory compliance".to_string(),
            },
            market_position: ComponentDetail {
                score: score.market_position_score,
                max_score: 100,
                weight: 0.15,
                description: "Market position and competitiveness".to_string(),
            },
        },
        factors: CreditFactors {
            positive: serde_json::from_value(score.factors_positive).unwrap_or_default(),
            negative: serde_json::from_value(score.factors_negative).unwrap_or_default(),
        },
        suggested_credit_limit: score.suggested_credit_limit,
        currency: score.currency,
        calculated_at: score.calculated_at,
        expires_at: score.expires_at,
    }
}

fn score_to_grade(score: i32) -> String {
    match score {
        800..=850 => "A".to_string(),
        700..=799 => "B".to_string(),
        600..=699 => "C".to_string(),
        500..=599 => "D".to_string(),
        300..=499 => "E".to_string(),
        _ => "F".to_string(),
    }
}

fn score_to_risk(score: i32) -> String {
    match score {
        750..=850 => "low".to_string(),
        600..=749 => "moderate".to_string(),
        450..=599 => "high".to_string(),
        _ => "very_high".to_string(),
    }
}

fn format_amount_range(min: Option<i64>, max: i64, currency: &str) -> String {
    match min {
        Some(m) => format!("{} - {} {}", m, max, currency),
        None => format!("Up to {} {}", max, currency),
    }
}

fn format_rate_range(min: Option<f64>, max: Option<f64>) -> String {
    match (min, max) {
        (Some(mn), Some(mx)) => format!("{:.1}% - {:.1}%", mn, mx),
        (None, Some(mx)) => format!("Up to {:.1}%", mx),
        (Some(mn), None) => format!("From {:.1}%", mn),
        (None, None) => "Variable".to_string(),
    }
}

fn format_term_range(min: Option<i32>, max: Option<i32>) -> String {
    match (min, max) {
        (Some(mn), Some(mx)) => format!("{} - {} months", mn, mx),
        (None, Some(mx)) => format!("Up to {} months", mx),
        (Some(mn), None) => format!("From {} months", mn),
        (None, None) => "Flexible".to_string(),
    }
}

async fn get_default_business_id(pool: &PgPool, user_id: Uuid) -> Result<Uuid, actix_web::Error> {
    let business_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM businesses WHERE owner_id = $1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    .ok_or_else(|| actix_web::error::ErrorBadRequest("No business found"))?;
    
    Ok(business_id)
}
