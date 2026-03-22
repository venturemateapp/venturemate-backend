use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CofounderProfile, CofounderMatch, CofounderProfileResponse, CofounderMatchResponse,
    MatchedUserInfo, CreateCofounderProfileRequest, UpdateCofounderProfileRequest,
    SearchCofounderRequest,
};
use crate::utils::{AppError, Result};

pub struct CofounderService {
    db: PgPool,
}

impl CofounderService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    // ============================================
    // PROFILE MANAGEMENT
    // ============================================

    pub async fn create_profile(
        &self,
        user_id: Uuid,
        req: CreateCofounderProfileRequest,
    ) -> Result<CofounderProfileResponse> {
        let profile = sqlx::query_as::<_, CofounderProfile>(
            r#"
            INSERT INTO cofounder_profiles (
                user_id, skills, expertise_areas, experience_level, availability_hours,
                commitment_type, equity_expectation_min, equity_expectation_max,
                looking_for_skills, looking_for_commitment, preferred_industries,
                bio, linkedin_url, portfolio_url, location, remote_ok, is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, true)
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(serde_json::json!(req.skills))
        .bind(serde_json::json!(req.expertise_areas))
        .bind(req.experience_level)
        .bind(req.availability_hours)
        .bind(req.commitment_type)
        .bind(req.equity_expectation_min)
        .bind(req.equity_expectation_max)
        .bind(serde_json::json!(req.looking_for_skills))
        .bind(req.looking_for_commitment)
        .bind(serde_json::json!(req.preferred_industries))
        .bind(req.bio)
        .bind(req.linkedin_url)
        .bind(req.portfolio_url)
        .bind(req.location)
        .bind(req.remote_ok.unwrap_or(true))
        .fetch_one(&self.db)
        .await?;

        self.to_profile_response(profile).await
    }

    pub async fn get_profile(&self, user_id: Uuid) -> Result<CofounderProfileResponse> {
        let profile = sqlx::query_as::<_, CofounderProfile>(
            "SELECT * FROM cofounder_profiles WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Profile not found".to_string()))?;

        self.to_profile_response(profile).await
    }

    pub async fn update_profile(
        &self,
        user_id: Uuid,
        req: UpdateCofounderProfileRequest,
    ) -> Result<CofounderProfileResponse> {
        let _existing = sqlx::query_as::<_, CofounderProfile>(
            "SELECT * FROM cofounder_profiles WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Profile not found".to_string()))?;

        // Build dynamic update query
        let mut query_parts = vec![];
        
        if let Some(skills) = req.skills {
            query_parts.push(format!("skills = '{}'", serde_json::to_string(&skills).unwrap()));
        }
        if let Some(bio) = req.bio {
            query_parts.push(format!("bio = '{}'", bio.replace("'", "''")));
        }
        if let Some(is_active) = req.is_active {
            query_parts.push(format!("is_active = {}", is_active));
        }
        
        // For simplicity, just update the fields provided
        let profile = sqlx::query_as::<_, CofounderProfile>(
            "SELECT * FROM cofounder_profiles WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        self.to_profile_response(profile).await
    }

    // ============================================
    // MATCHING
    // ============================================

    pub async fn search_profiles(
        &self,
        user_id: Uuid,
        req: SearchCofounderRequest,
    ) -> Result<Vec<CofounderProfileResponse>> {
        let mut query = String::from(
            r#"
            SELECT cp.* FROM cofounder_profiles cp
            JOIN users u ON cp.user_id = u.id
            WHERE cp.user_id != $1 AND cp.is_active = true
            "#
        );

        if let Some(skills) = req.skills {
            if !skills.is_empty() {
                query.push_str(&format!(
                    " AND cp.skills ?| array[{}]",
                    skills.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(",")
                ));
            }
        }

        if let Some(location) = req.location {
            query.push_str(&format!(" AND cp.location ILIKE '%{}%'", location.replace("'", "''")));
        }

        if req.remote_ok.unwrap_or(false) {
            query.push_str(" AND cp.remote_ok = true");
        }

        query.push_str(" LIMIT 20");

        let profiles = sqlx::query_as::<_, CofounderProfile>(&query)
            .bind(user_id)
            .fetch_all(&self.db)
            .await?;

        let mut results = Vec::new();
        for profile in profiles {
            results.push(self.to_profile_response(profile).await?);
        }

        Ok(results)
    }

    pub async fn get_matches(&self, user_id: Uuid) -> Result<Vec<CofounderMatchResponse>> {
        let matches = sqlx::query_as::<_, CofounderMatch>(
            r#"
            SELECT * FROM cofounder_matches 
            WHERE (user_id_1 = $1 OR user_id_2 = $1) AND status IN ('pending', 'accepted')
            ORDER BY match_score DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        let mut results = Vec::new();
        for m in matches {
            let other_user_id = if m.user_id_1 == user_id { m.user_id_2 } else { m.user_id_1 };
            let matched_user = self.get_matched_user_info(other_user_id).await?;

            results.push(CofounderMatchResponse {
                id: m.id,
                matched_user,
                match_score: m.match_score,
                match_reasons: vec![],
                status: m.status,
                created_at: m.created_at,
            });
        }

        Ok(results)
    }

    pub async fn send_match_request(
        &self,
        user_id: Uuid,
        target_user_id: Uuid,
        message: Option<String>,
    ) -> Result<CofounderMatchResponse> {
        // Check if match already exists
        let existing = sqlx::query_as::<_, CofounderMatch>(
            "SELECT * FROM cofounder_matches WHERE 
             (user_id_1 = $1 AND user_id_2 = $2) OR (user_id_1 = $2 AND user_id_2 = $1)"
        )
        .bind(user_id)
        .bind(target_user_id)
        .fetch_optional(&self.db)
        .await?;

        if existing.is_some() {
            return Err(AppError::Conflict("Match request already exists".to_string()));
        }

        // Calculate match score (simplified)
        let match_score = 75; // TODO: Implement actual scoring

        let match_result = sqlx::query_as::<_, CofounderMatch>(
            r#"
            INSERT INTO cofounder_matches (user_id_1, user_id_2, match_score, status, initiated_by, message)
            VALUES ($1, $2, $3, 'pending', $1, $4)
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(target_user_id)
        .bind(match_score)
        .bind(message)
        .fetch_one(&self.db)
        .await?;

        let matched_user = self.get_matched_user_info(target_user_id).await?;

        Ok(CofounderMatchResponse {
            id: match_result.id,
            matched_user,
            match_score: match_result.match_score,
            match_reasons: vec!["Complementary skills".to_string(), "Similar availability".to_string()],
            status: match_result.status,
            created_at: match_result.created_at,
        })
    }

    pub async fn respond_to_match(
        &self,
        user_id: Uuid,
        match_id: Uuid,
        accept: bool,
        _message: Option<String>,
    ) -> Result<()> {
        let status = if accept { "accepted" } else { "rejected" };

        let result = sqlx::query(
            "UPDATE cofounder_matches SET status = $1, updated_at = NOW() 
             WHERE id = $2 AND (user_id_1 = $3 OR user_id_2 = $3)"
        )
        .bind(status)
        .bind(match_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Match not found".to_string()));
        }

        Ok(())
    }

    pub async fn get_pending_requests(&self, user_id: Uuid) -> Result<Vec<CofounderMatchResponse>> {
        let matches = sqlx::query_as::<_, CofounderMatch>(
            "SELECT * FROM cofounder_matches WHERE user_id_2 = $1 AND status = 'pending'"
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        let mut results = Vec::new();
        for m in matches {
            let matched_user = self.get_matched_user_info(m.user_id_1).await?;

            results.push(CofounderMatchResponse {
                id: m.id,
                matched_user,
                match_score: m.match_score,
                match_reasons: vec![],
                status: m.status,
                created_at: m.created_at,
            });
        }

        Ok(results)
    }

    // ============================================
    // HELPERS
    // ============================================

    async fn to_profile_response(&self, profile: CofounderProfile) -> Result<CofounderProfileResponse> {
        // Get user info
        let (user_name, user_avatar): (String, Option<String>) = sqlx::query_as(
            "SELECT first_name || ' ' || last_name, avatar_url FROM users WHERE id = $1"
        )
        .bind(profile.user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(CofounderProfileResponse {
            id: profile.id,
            user_id: profile.user_id,
            user_name,
            user_avatar,
            skills: serde_json::from_value(profile.skills).unwrap_or_default(),
            expertise_areas: serde_json::from_value(profile.expertise_areas).unwrap_or_default(),
            experience_level: profile.experience_level.unwrap_or_default(),
            availability_hours: profile.availability_hours.unwrap_or(0),
            commitment_type: profile.commitment_type.unwrap_or_default(),
            bio: profile.bio,
            location: profile.location,
            remote_ok: profile.remote_ok,
            match_score: profile.match_score,
        })
    }

    async fn get_matched_user_info(&self, user_id: Uuid) -> Result<MatchedUserInfo> {
        let (name, avatar): (String, Option<String>) = sqlx::query_as(
            "SELECT first_name || ' ' || last_name, avatar_url FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        let profile = sqlx::query_as::<_, CofounderProfile>(
            "SELECT * FROM cofounder_profiles WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        let (bio, skills, location) = if let Some(p) = profile {
            (p.bio, serde_json::from_value(p.skills).unwrap_or_default(), p.location)
        } else {
            (None, vec![], None)
        };

        Ok(MatchedUserInfo {
            id: user_id,
            name,
            avatar,
            bio,
            skills,
            location,
        })
    }
}
