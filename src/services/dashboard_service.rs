//! Founder Dashboard Service
//! Aggregates all data for the dashboard view

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    DashboardData, NextAction, QuickStats, StartupActivity, StartupOverview,
    StartupProgressResponse, UpcomingDeadline,
};
use crate::utils::Result;

pub struct DashboardService {
    db: PgPool,
}

impl DashboardService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Get complete dashboard data for a startup
    pub async fn get_dashboard(&self, startup_id: Uuid, user_id: Uuid) -> Result<DashboardData> {
        // Verify ownership
        let startup: StartupOverview = sqlx::query_as(
            r#"
            SELECT so.* FROM startup_overview so
            JOIN startups s ON so.id = s.id
            WHERE so.id = $1 AND s.user_id = $2
            "#,
        )
        .bind(startup_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        // Get progress
        let progress = self.get_progress(startup_id).await?;

        // Get next actions
        let next_actions = self.get_next_actions(startup_id).await?;

        // Get activity feed
        let activity_feed = self.get_activity_feed(startup_id).await?;

        // Get upcoming deadlines
        let upcoming_deadlines = self.get_upcoming_deadlines(startup_id).await?;

        // Get quick stats
        let quick_stats = self.get_quick_stats(startup_id).await?;

        Ok(DashboardData {
            startup,
            health_score: progress.health_score.unwrap_or(10),
            progress,
            next_actions,
            activity_feed,
            upcoming_deadlines,
            quick_stats,
        })
    }

    async fn get_progress(&self, startup_id: Uuid) -> Result<StartupProgressResponse> {
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

    async fn get_next_actions(&self, startup_id: Uuid) -> Result<Vec<NextAction>> {
        let mut actions = vec![];

        // Priority 1: Milestones that are pending or in_progress
        let milestone_actions: Vec<NextAction> = sqlx::query_as(
            r#"
            SELECT 
                m.id::text as action_id,
                'milestone' as action_type,
                m.title,
                m.description as description,
                CASE 
                    WHEN m.due_date < NOW() + INTERVAL '7 days' THEN 1
                    WHEN m.due_date < NOW() + INTERVAL '14 days' THEN 2
                    ELSE 3
                END as priority,
                m.due_date::text as due_date,
                m.status,
                '/milestones/' || m.id::text as action_url,
                m.estimated_days::text as metadata
            FROM milestones m
            WHERE m.startup_id = $1
            AND m.status IN ('pending', 'in_progress')
            AND (m.depends_on_milestones = '[]' OR m.depends_on_milestones IS NULL
                 OR NOT EXISTS (
                     SELECT 1 FROM milestones dep
                     WHERE dep.id::text = ANY(ARRAY(
                         SELECT jsonb_array_elements_text(m.depends_on_milestones)
                     ))
                     AND dep.status != 'completed'
                 ))
            ORDER BY priority ASC, m.order_sequence ASC
            LIMIT 5
            "#,
        )
        .bind(startup_id)
        .fetch_all(&self.db)
        .await?;

        actions.extend(milestone_actions);

        // Priority 2: Approvals that need attention
        let approval_actions: Vec<NextAction> = sqlx::query_as(
            r#"
            SELECT 
                ra.id::text as action_id,
                'approval' as action_type,
                ra.name as title,
                ra.description,
                ra.priority as priority,
                ra.submission_date::text as due_date,
                ra.status,
                '/approvals/' || ra.id::text as action_url,
                ra.documents_required::text as metadata
            FROM required_approvals ra
            WHERE ra.startup_id = $1
            AND ra.status IN ('not_started', 'in_progress')
            ORDER BY ra.priority ASC
            LIMIT 3
            "#,
        )
        .bind(startup_id)
        .fetch_all(&self.db)
        .await?;

        actions.extend(approval_actions);

        // Sort by priority and limit
        actions.sort_by(|a, b| a.priority.cmp(&b.priority));
        actions.truncate(5);

        Ok(actions)
    }

    async fn get_activity_feed(&self, startup_id: Uuid) -> Result<Vec<StartupActivity>> {
        let activities: Vec<StartupActivity> = sqlx::query_as(
            r#"
            SELECT 
                'milestone_completed' as activity_type,
                m.title as description,
                m.completed_at as occurred_at,
                jsonb_build_object('milestone_id', m.id) as metadata
            FROM milestones m
            WHERE m.startup_id = $1 AND m.status = 'completed' AND m.completed_at IS NOT NULL
            
            UNION ALL
            
            SELECT 
                'approval_updated' as activity_type,
                ra.name || ' status changed to ' || ra.status as description,
                ra.updated_at as occurred_at,
                jsonb_build_object('approval_id', ra.id, 'status', ra.status) as metadata
            FROM required_approvals ra
            WHERE ra.startup_id = $1 AND ra.updated_at > ra.created_at
            
            UNION ALL
            
            SELECT 
                'service_connected' as activity_type,
                ss.service_name || ' connected' as description,
                ss.connected_at as occurred_at,
                jsonb_build_object('service_id', ss.id) as metadata
            FROM suggested_services ss
            WHERE ss.startup_id = $1 AND ss.status = 'connected' AND ss.connected_at IS NOT NULL
            
            ORDER BY occurred_at DESC
            LIMIT 10
            "#,
        )
        .bind(startup_id)
        .fetch_all(&self.db)
        .await?;

        Ok(activities)
    }

    async fn get_upcoming_deadlines(&self, startup_id: Uuid) -> Result<Vec<crate::models::UpcomingDeadline>> {
        let deadlines: Vec<crate::models::UpcomingDeadline> = sqlx::query_as(
            r#"
            SELECT 
                s.id as startup_id,
                s.name as startup_name,
                m.id as milestone_id,
                m.title as milestone_title,
                m.due_date,
                m.status,
                CASE 
                    WHEN m.due_date < NOW() THEN 'overdue'
                    WHEN m.due_date < NOW() + INTERVAL '7 days' THEN 'this_week'
                    ELSE 'upcoming'
                END as urgency
            FROM startups s
            JOIN milestones m ON s.id = m.startup_id
            WHERE s.id = $1
            AND m.status NOT IN ('completed', 'skipped')
            AND m.due_date IS NOT NULL
            ORDER BY m.due_date ASC
            LIMIT 5
            "#,
        )
        .bind(startup_id)
        .fetch_all(&self.db)
        .await?;

        Ok(deadlines)
    }

    async fn get_quick_stats(&self, startup_id: Uuid) -> Result<crate::models::QuickStats> {
        let stats: crate::models::QuickStats = sqlx::query_as(
            r#"
            SELECT 
                COUNT(DISTINCT ra.id) FILTER (WHERE ra.status = 'approved') as approvals_completed,
                COUNT(DISTINCT ra.id) as approvals_total,
                COUNT(DISTINCT sd.id) FILTER (WHERE sd.status = 'ready') as documents_uploaded,
                COUNT(DISTINCT sd.id) as documents_total,
                COUNT(DISTINCT ss.id) FILTER (WHERE ss.status = 'connected') as services_connected
            FROM startups s
            LEFT JOIN required_approvals ra ON s.id = ra.startup_id
            LEFT JOIN startup_documents sd ON s.id = sd.startup_id
            LEFT JOIN suggested_services ss ON s.id = ss.startup_id
            WHERE s.id = $1
            GROUP BY s.id
            "#,
        )
        .bind(startup_id)
        .fetch_one(&self.db)
        .await?;

        Ok(stats)
    }
}
