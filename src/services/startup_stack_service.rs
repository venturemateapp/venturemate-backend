//! Startup Stack Generator Service
//! Transforms AI blueprint into structured database records

use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    AiBlueprint, ApprovalResponse, ConnectServiceRequest, CreateStartupRequest,
    GenerateStartupStackResponse, Milestone,
    MilestoneResponse, RequiredApproval, Startup, StartupDocument,
    StartupMetric, StartupOverview, StartupProgressResponse, StartupResponse,
    SuggestedService, UpdateApprovalRequest, UpdateMilestoneRequest, UpdateStartupRequest,
    UpcomingDeadline,
};
use crate::utils::{AppError, Result};

pub struct StartupStackService {
    db: PgPool,
}

impl StartupStackService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    // ============================================
    // 1. CREATE STARTUP FROM BLUEPRINT
    // ============================================
    pub async fn create_from_blueprint(
        &self,
        user_id: Uuid,
        blueprint: AiBlueprint,
    ) -> Result<GenerateStartupStackResponse> {
        // Validate blueprint
        self.validate_blueprint(&blueprint).await?;

        // Create startup record
        let startup_id = self.create_startup_record(user_id, &blueprint).await?;

        // Generate milestones
        let milestones_created = self.generate_milestones(startup_id, &blueprint).await?;

        // Generate approvals
        let approvals_created = self.generate_approvals(startup_id, &blueprint).await?;

        // Generate suggested services
        let services_suggested = self.generate_services(startup_id, &blueprint).await?;

        // Queue document generation
        let documents_queued = self.queue_documents(startup_id, &blueprint).await?;

        // Calculate initial health score
        self.calculate_health_score(startup_id).await?;

        Ok(GenerateStartupStackResponse {
            startup_id,
            milestones_created,
            approvals_created,
            services_suggested,
            documents_queued,
        })
    }

    async fn validate_blueprint(&self, blueprint: &AiBlueprint) -> Result<()> {
        // Business name must be present
        if blueprint.business_name.trim().is_empty() {
            return Err(AppError::Validation("Business name is required".to_string()));
        }

        // Industry validation
        let valid_industries = vec![
            "fintech", "agriculture", "healthtech", "edtech", "ecommerce",
            "saas", "logistics", "entertainment", "manufacturing", "other",
        ];
        if !valid_industries.contains(&blueprint.industry.to_lowercase().as_str()) {
            // Allow but log warning
            tracing::warn!("Unrecognized industry: {}", blueprint.industry);
        }

        // Country validation
        let country_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM supported_countries WHERE code = $1)"
        )
        .bind(&blueprint.country.to_uppercase())
        .fetch_one(&self.db)
        .await?;

        if !country_exists {
            return Err(AppError::Validation(
                format!("Country {} is not supported", blueprint.country)
            ));
        }

        // Must have at least some milestones
        if blueprint.milestones.is_empty() {
            return Err(AppError::Validation(
                "At least one milestone is required".to_string()
            ));
        }

        Ok(())
    }

    async fn create_startup_record(
        &self,
        user_id: Uuid,
        blueprint: &AiBlueprint,
    ) -> Result<Uuid> {
        let alternative_names = json!(blueprint.alternative_names);
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO startups (
                id, user_id, name, alternative_names, tagline, elevator_pitch,
                mission_statement, vision_statement, industry, sub_industry,
                country, founder_type, business_stage, status, progress_percentage, health_score
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'solo', 'idea', 'active', 0, 10)
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(&blueprint.business_name)
        .bind(&alternative_names)
        .bind(&blueprint.tagline)
        .bind(&blueprint.elevator_pitch)
        .bind(&blueprint.mission_statement)
        .bind(&blueprint.vision_statement)
        .bind(&blueprint.industry)
        .bind(&blueprint.sub_industry)
        .bind(&blueprint.country.to_uppercase())
        .fetch_one(&self.db)
        .await?;

        // Log metric
        sqlx::query(
            "INSERT INTO startup_metrics (startup_id, metric_type, metric_value) VALUES ($1, 'health_score', 10)"
        )
        .bind(id)
        .execute(&self.db)
        .await?;

        Ok(id)
    }

    async fn generate_milestones(
        &self,
        startup_id: Uuid,
        blueprint: &AiBlueprint,
    ) -> Result<usize> {
        let mut count = 0;
        let mut current_sequence = 0;

        // Start with default milestones
        let default_milestones: Vec<(String, String, i32)> = vec![
            ("Validate Business Idea".to_string(), "Confirm your business idea is viable and has market potential".to_string(), 2),
            ("Register Business Entity".to_string(), "Register your business with the appropriate government authority".to_string(), 7),
            ("Get Tax Identification Number".to_string(), "Obtain your TIN for tax compliance".to_string(), 5),
            ("Open Business Bank Account".to_string(), "Set up a dedicated business banking account".to_string(), 7),
            ("Create Brand Identity".to_string(), "Design logo, choose colors, establish brand guidelines".to_string(), 5),
            ("Build Website/MVP".to_string(), "Create your online presence or minimum viable product".to_string(), 21),
            ("Set Up Payment Processing".to_string(), "Enable customers to pay you online".to_string(), 3),
            ("Create Marketing Strategy".to_string(), "Plan how you will acquire customers".to_string(), 7),
        ];

        let mut cumulative_days = 0;

        for (title, description, days) in default_milestones {
            current_sequence += 10;
            cumulative_days += days;
            let due_date = Utc::now() + Duration::days(cumulative_days as i64);

            sqlx::query(
                r#"
                INSERT INTO milestones (
                    startup_id, title, description, category, order_sequence,
                    estimated_days, status, completion_criteria, due_date
                )
                VALUES ($1, $2, $3, $4, $5, $6, 'pending', '{}', $7)
                "#,
            )
            .bind(startup_id)
            .bind(&title)
            .bind(&description)
            .bind("general")
            .bind(current_sequence)
            .bind(days)
            .bind(due_date)
            .execute(&self.db)
            .await?;

            count += 1;
        }

        // Add AI-suggested milestones
        for (idx, ai_milestone) in blueprint.milestones.iter().enumerate() {
            current_sequence += 10;
            let days = ai_milestone.estimated_days.max(1);
            cumulative_days += days;
            let due_date = Utc::now() + Duration::days(cumulative_days as i64);
            let deps = json!(ai_milestone.dependencies);

            sqlx::query(
                r#"
                INSERT INTO milestones (
                    startup_id, title, description, category, order_sequence,
                    estimated_days, status, completion_criteria, depends_on_milestones, due_date
                )
                VALUES ($1, $2, $3, $4, $5, $6, 'pending', '{}', $7, $8)
                "#,
            )
            .bind(startup_id)
            .bind(&ai_milestone.title)
            .bind(&ai_milestone.description)
            .bind(&ai_milestone.category)
            .bind(current_sequence)
            .bind(days)
            .bind(&deps)
            .bind(due_date)
            .execute(&self.db)
            .await?;

            count += 1;
        }

        Ok(count)
    }

    async fn generate_approvals(
        &self,
        startup_id: Uuid,
        blueprint: &AiBlueprint,
    ) -> Result<usize> {
        let mut count = 0;

        // Country-specific default approvals
        let country = blueprint.country.to_uppercase();
        
        if country == "NG" {
            // Nigeria-specific approvals
            let nigeria_approvals = vec![
                ("CAC Business Registration", "Corporate Affairs Commission", "registration", 7, vec!["ID Proof", "Business Address"]),
                ("TIN Registration", "Federal Inland Revenue Service", "tax", 5, vec!["CAC Certificate"]),
                ("VAT Registration", "Federal Inland Revenue Service", "tax", 3, vec!["TIN Certificate"]),
            ];

            for (idx, (name, authority, approval_type, days, docs)) in nigeria_approvals.iter().enumerate() {
                let docs_json = json!(docs);
                
                sqlx::query(
                    r#"
                    INSERT INTO required_approvals (
                        startup_id, approval_type, name, issuing_authority,
                        description, status, priority, estimated_days, documents_required
                    )
                    VALUES ($1, $2, $3, $4, $5, 'not_started', $6, $7, $8)
                    "#,
                )
                .bind(startup_id)
                .bind(approval_type)
                .bind(name)
                .bind(authority)
                .bind(format!("{} approval required for business operations in Nigeria", name))
                .bind((idx + 1) as i32)
                .bind(days)
                .bind(&docs_json)
                .execute(&self.db)
                .await?;

                count += 1;
            }
        } else {
            // Generic approvals for other countries
            let generic_approvals = vec![
                ("Business Registration", "Local Authority", "registration", 10),
                ("Tax Registration", "Tax Authority", "tax", 7),
            ];

            for (idx, (name, authority, approval_type, days)) in generic_approvals.iter().enumerate() {
                sqlx::query(
                    r#"
                    INSERT INTO required_approvals (
                        startup_id, approval_type, name, issuing_authority,
                        description, status, priority, estimated_days, documents_required
                    )
                    VALUES ($1, $2, $3, $4, $5, 'not_started', $6, $7, '[]')
                    "#,
                )
                .bind(startup_id)
                .bind(approval_type)
                .bind(name)
                .bind(authority)
                .bind(format!("{} required for business operations", name))
                .bind((idx + 1) as i32)
                .bind(days)
                .execute(&self.db)
                .await?;

                count += 1;
            }
        }

        // Add AI-suggested approvals
        for (idx, ai_approval) in blueprint.approvals.iter().enumerate() {
            let docs_json = json!(ai_approval.documents_required);
            
            sqlx::query(
                r#"
                INSERT INTO required_approvals (
                    startup_id, approval_type, name, issuing_authority,
                    description, status, priority, estimated_days, documents_required
                )
                VALUES ($1, $2, $3, $4, $5, 'not_started', $6, $7, $8)
                "#,
            )
            .bind(startup_id)
            .bind(&ai_approval.approval_type)
            .bind(&ai_approval.name)
            .bind(&ai_approval.issuing_authority)
            .bind(format!("{} approval required", ai_approval.name))
            .bind((idx + 10) as i32)
            .bind(ai_approval.estimated_days)
            .bind(&docs_json)
            .execute(&self.db)
            .await?;

            count += 1;
        }

        Ok(count)
    }

    async fn generate_services(
        &self,
        startup_id: Uuid,
        blueprint: &AiBlueprint,
    ) -> Result<usize> {
        // Get service templates from database
        let templates: Vec<SuggestedService> = sqlx::query_as(
            r#"
            SELECT 
                gen_random_uuid() as id,
                $1 as startup_id,
                service_category, service_name, service_provider, description,
                features, pricing_model, price_range, website_url, integration_type,
                is_partner, partnership_benefits, priority, 'suggested' as status,
                NULL as connected_at, NOW() as created_at, NOW() as updated_at
            FROM service_templates
            WHERE applicable_countries @> $2 OR applicable_countries @> '["all"]'
            ORDER BY priority ASC
            LIMIT 15
            "#,
        )
        .bind(startup_id)
        .bind(json!(vec![blueprint.country.to_uppercase()]))
        .fetch_all(&self.db)
        .await?;

        let count = templates.len();

        // Insert suggested services
        for service in templates {
            sqlx::query(
                r#"
                INSERT INTO suggested_services (
                    startup_id, service_category, service_name, service_provider,
                    description, features, pricing_model, price_range, website_url,
                    integration_type, is_partner, partnership_benefits, priority, status
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 'suggested')
                "#,
            )
            .bind(startup_id)
            .bind(&service.service_category)
            .bind(&service.service_name)
            .bind(&service.service_provider)
            .bind(&service.description)
            .bind(&service.features)
            .bind(&service.pricing_model)
            .bind(&service.price_range)
            .bind(&service.website_url)
            .bind(&service.integration_type)
            .bind(service.is_partner)
            .bind(&service.partnership_benefits)
            .bind(service.priority)
            .execute(&self.db)
            .await?;
        }

        Ok(count)
    }

    async fn queue_documents(
        &self,
        startup_id: Uuid,
        _blueprint: &AiBlueprint,
    ) -> Result<usize> {
        let documents = vec![
            ("business_plan", "Business Plan"),
            ("pitch_deck", "Pitch Deck"),
            ("brand_kit", "Brand Identity Kit"),
        ];
        let doc_count = documents.len();

        for (doc_type, name) in &documents {
            sqlx::query(
                r#"
                INSERT INTO startup_documents (
                    startup_id, document_type, document_name, status
                )
                VALUES ($1, $2, $3, 'generating')
                "#,
            )
            .bind(startup_id)
            .bind(doc_type)
            .bind(name)
            .execute(&self.db)
            .await?;
        }

        Ok(doc_count)
    }

    // ============================================
    // 2. READ OPERATIONS
    // ============================================
    pub async fn get_startup(&self, startup_id: Uuid, user_id: Uuid) -> Result<StartupResponse> {
        let startup: Startup = sqlx::query_as(
            "SELECT * FROM startups WHERE id = $1 AND user_id = $2"
        )
        .bind(startup_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(startup.into())
    }

    pub async fn list_startups(&self, user_id: Uuid) -> Result<Vec<StartupOverview>> {
        let startups: Vec<StartupOverview> = sqlx::query_as(
            "SELECT * FROM startup_overview WHERE id IN (SELECT id FROM startups WHERE user_id = $1)"
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(startups)
    }

    pub async fn get_milestones(&self, startup_id: Uuid, status: Option<String>) -> Result<Vec<MilestoneResponse>> {
        let query = if let Some(s) = status {
            sqlx::query_as::<_, Milestone>(
                "SELECT * FROM milestones WHERE startup_id = $1 AND status = $2 ORDER BY order_sequence ASC"
            )
            .bind(startup_id)
            .bind(s)
        } else {
            sqlx::query_as::<_, Milestone>(
                "SELECT * FROM milestones WHERE startup_id = $1 ORDER BY order_sequence ASC"
            )
            .bind(startup_id)
        };

        let milestones: Vec<Milestone> = query.fetch_all(&self.db).await?;
        Ok(milestones.into_iter().map(Into::into).collect())
    }

    pub async fn get_approvals(&self, startup_id: Uuid) -> Result<Vec<ApprovalResponse>> {
        let approvals: Vec<RequiredApproval> = sqlx::query_as(
            "SELECT * FROM required_approvals WHERE startup_id = $1 ORDER BY priority ASC"
        )
        .bind(startup_id)
        .fetch_all(&self.db)
        .await?;

        Ok(approvals.into_iter().map(Into::into).collect())
    }

    pub async fn get_services(&self, startup_id: Uuid, category: Option<String>) -> Result<Vec<crate::models::StartupServiceResponse>> {
        let query = if let Some(c) = category {
            sqlx::query_as::<_, SuggestedService>(
                "SELECT * FROM suggested_services WHERE startup_id = $1 AND service_category = $2 ORDER BY priority ASC"
            )
            .bind(startup_id)
            .bind(c)
        } else {
            sqlx::query_as::<_, SuggestedService>(
                "SELECT * FROM suggested_services WHERE startup_id = $1 ORDER BY priority ASC"
            )
            .bind(startup_id)
        };

        let services: Vec<SuggestedService> = query.fetch_all(&self.db).await?;
        Ok(services.into_iter().map(Into::into).collect())
    }

    pub async fn get_documents(&self, startup_id: Uuid) -> Result<Vec<crate::models::StartupDocumentResponse>> {
        let documents: Vec<StartupDocument> = sqlx::query_as(
            "SELECT * FROM startup_documents WHERE startup_id = $1 ORDER BY created_at DESC"
        )
        .bind(startup_id)
        .fetch_all(&self.db)
        .await?;

        Ok(documents.into_iter().map(|d| crate::models::StartupDocumentResponse {
            id: d.id,
            document_type: d.document_type,
            document_name: d.document_name,
            file_url: d.file_url,
            version: d.version,
            status: d.status,
            generated_at: d.generated_at,
        }).collect())
    }

    pub async fn get_progress(&self, startup_id: Uuid) -> Result<StartupProgressResponse> {
        let progress: StartupProgressResponse = sqlx::query_as(
            r#"
            SELECT 
                s.id as startup_id,
                s.progress_percentage as overall_percentage,
                COUNT(DISTINCT m.id) FILTER (WHERE m.status = 'completed') as completed_milestones,
                COUNT(DISTINCT m.id) as total_milestones,
                COUNT(DISTINCT ra.id) FILTER (WHERE ra.status = 'approved') as completed_approvals,
                COUNT(DISTINCT ra.id) as total_approvals,
                COUNT(DISTINCT ss.id) FILTER (WHERE ss.status = 'connected') as connected_services,
                COUNT(DISTINCT ss.id) as total_services,
                s.health_score
            FROM startups s
            LEFT JOIN milestones m ON s.id = m.startup_id
            LEFT JOIN required_approvals ra ON s.id = ra.startup_id
            LEFT JOIN suggested_services ss ON s.id = ss.startup_id
            WHERE s.id = $1
            GROUP BY s.id, s.progress_percentage, s.health_score
            "#,
        )
        .bind(startup_id)
        .fetch_one(&self.db)
        .await?;

        Ok(progress)
    }

    pub async fn get_upcoming_deadlines(&self, user_id: Uuid) -> Result<Vec<UpcomingDeadline>> {
        let deadlines: Vec<UpcomingDeadline> = sqlx::query_as(
            r#"
            SELECT ud.* 
            FROM upcoming_deadlines ud
            JOIN startups s ON ud.startup_id = s.id
            WHERE s.user_id = $1
            AND ud.urgency IN ('overdue', 'this_week')
            ORDER BY ud.due_date ASC
            LIMIT 10
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(deadlines)
    }

    // ============================================
    // 3. UPDATE OPERATIONS
    // ============================================
    pub async fn update_startup(
        &self,
        startup_id: Uuid,
        user_id: Uuid,
        req: UpdateStartupRequest,
    ) -> Result<StartupResponse> {
        sqlx::query(
            r#"
            UPDATE startups SET
                name = COALESCE($1, name),
                tagline = COALESCE($2, tagline),
                elevator_pitch = COALESCE($3, elevator_pitch),
                mission_statement = COALESCE($4, mission_statement),
                vision_statement = COALESCE($5, vision_statement),
                industry = COALESCE($6, industry),
                sub_industry = COALESCE($7, sub_industry),
                business_stage = COALESCE($8, business_stage),
                status = COALESCE($9, status),
                updated_at = NOW()
            WHERE id = $10 AND user_id = $11
            "#,
        )
        .bind(&req.name)
        .bind(&req.tagline)
        .bind(&req.elevator_pitch)
        .bind(&req.mission_statement)
        .bind(&req.vision_statement)
        .bind(&req.industry)
        .bind(&req.sub_industry)
        .bind(&req.business_stage)
        .bind(&req.status)
        .bind(startup_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        self.get_startup(startup_id, user_id).await
    }

    pub async fn update_milestone(
        &self,
        milestone_id: Uuid,
        startup_id: Uuid,
        req: UpdateMilestoneRequest,
    ) -> Result<MilestoneResponse> {
        sqlx::query(
            r#"
            UPDATE milestones SET
                status = COALESCE($1, status),
                started_at = COALESCE($2, started_at),
                completed_at = COALESCE($3, completed_at),
                updated_at = NOW()
            WHERE id = $4 AND startup_id = $5
            "#,
        )
        .bind(&req.status)
        .bind(req.started_at)
        .bind(req.completed_at)
        .bind(milestone_id)
        .bind(startup_id)
        .execute(&self.db)
        .await?;

        // Recalculate progress
        self.recalculate_progress(startup_id).await?;

        let milestone: Milestone = sqlx::query_as("SELECT * FROM milestones WHERE id = $1")
            .bind(milestone_id)
            .fetch_one(&self.db)
            .await?;

        Ok(milestone.into())
    }

    pub async fn update_approval(
        &self,
        approval_id: Uuid,
        startup_id: Uuid,
        req: UpdateApprovalRequest,
    ) -> Result<ApprovalResponse> {
        sqlx::query(
            r#"
            UPDATE required_approvals SET
                status = COALESCE($1, status),
                reference_number = COALESCE($2, reference_number),
                submission_date = COALESCE($3, submission_date),
                approval_date = COALESCE($4, approval_date),
                actual_cost = COALESCE($5, actual_cost),
                notes = COALESCE($6, notes),
                updated_at = NOW()
            WHERE id = $7 AND startup_id = $8
            "#,
        )
        .bind(&req.status)
        .bind(&req.reference_number)
        .bind(req.submission_date)
        .bind(req.approval_date)
        .bind(req.actual_cost)
        .bind(&req.notes)
        .bind(approval_id)
        .bind(startup_id)
        .execute(&self.db)
        .await?;

        // Recalculate health score
        self.calculate_health_score(startup_id).await?;

        let approval: RequiredApproval = sqlx::query_as("SELECT * FROM required_approvals WHERE id = $1")
            .bind(approval_id)
            .fetch_one(&self.db)
            .await?;

        Ok(approval.into())
    }

    pub async fn connect_service(
        &self,
        service_id: Uuid,
        startup_id: Uuid,
        _req: ConnectServiceRequest,
    ) -> Result<crate::models::StartupServiceResponse> {
        sqlx::query(
            r#"
            UPDATE suggested_services SET
                status = 'connected',
                connected_at = NOW(),
                updated_at = NOW()
            WHERE id = $1 AND startup_id = $2
            "#,
        )
        .bind(service_id)
        .bind(startup_id)
        .execute(&self.db)
        .await?;

        // Recalculate health score
        self.calculate_health_score(startup_id).await?;

        let service: SuggestedService = sqlx::query_as("SELECT * FROM suggested_services WHERE id = $1")
            .bind(service_id)
            .fetch_one(&self.db)
            .await?;

        Ok(service.into())
    }

    // ============================================
    // 4. PROGRESS & HEALTH CALCULATION
    // ============================================
    async fn recalculate_progress(&self, startup_id: Uuid) -> Result<()> {
        let completed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM milestones WHERE startup_id = $1 AND status = 'completed'"
        )
        .bind(startup_id)
        .fetch_one(&self.db)
        .await?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM milestones WHERE startup_id = $1"
        )
        .bind(startup_id)
        .fetch_one(&self.db)
        .await?;

        let progress = if total > 0 {
            ((completed as f64 / total as f64) * 100.0) as i32
        } else {
            0
        };

        sqlx::query("UPDATE startups SET progress_percentage = $1 WHERE id = $2")
            .bind(progress)
            .bind(startup_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    async fn calculate_health_score(&self, startup_id: Uuid) -> Result<i32> {
        // Get counts for each component
        let total_milestones: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM milestones WHERE startup_id = $1"
        )
        .bind(startup_id)
        .fetch_one(&self.db)
        .await?;

        let completed_milestones: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM milestones WHERE startup_id = $1 AND status = 'completed'"
        )
        .bind(startup_id)
        .fetch_one(&self.db)
        .await?;

        let total_approvals: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM required_approvals WHERE startup_id = $1"
        )
        .bind(startup_id)
        .fetch_one(&self.db)
        .await?;

        let completed_approvals: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM required_approvals WHERE startup_id = $1 AND status = 'approved'"
        )
        .bind(startup_id)
        .fetch_one(&self.db)
        .await?;

        let total_services: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM suggested_services WHERE startup_id = $1"
        )
        .bind(startup_id)
        .fetch_one(&self.db)
        .await?;

        let connected_services: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM suggested_services WHERE startup_id = $1 AND status = 'connected'"
        )
        .bind(startup_id)
        .fetch_one(&self.db)
        .await?;

        // Calculate component scores
        let compliance_score = if total_approvals > 0 {
            ((completed_approvals as f64 / total_approvals as f64) * 100.0) as i32
        } else {
            50 // Default if no approvals
        };

        let milestone_progress = if total_milestones > 0 {
            ((completed_milestones as f64 / total_milestones as f64) * 100.0) as i32
        } else {
            0
        };

        let service_integration = if total_services > 0 {
            ((connected_services as f64 / total_services as f64) * 100.0) as i32
        } else {
            0
        };

        // Calculate weighted health score
        // Compliance: 30%, Milestones: 40%, Services: 30%
        let health_score = (
            (compliance_score as f64 * 0.30) +
            (milestone_progress as f64 * 0.40) +
            (service_integration as f64 * 0.30)
        ) as i32;

        // Update startup
        sqlx::query("UPDATE startups SET health_score = $1 WHERE id = $2")
            .bind(health_score)
            .bind(startup_id)
            .execute(&self.db)
            .await?;

        // Log metric
        sqlx::query(
            "INSERT INTO startup_metrics (startup_id, metric_type, metric_value) VALUES ($1, 'health_score', $2)"
        )
        .bind(startup_id)
        .bind(health_score as f64)
        .execute(&self.db)
        .await?;

        Ok(health_score)
    }
}
