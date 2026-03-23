-- ============================================
-- Fix onboarding_answers unique constraint
-- Required for ON CONFLICT clause to work
-- ============================================

-- Add unique constraint if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_indexes 
        WHERE indexname = 'idx_onboarding_answers_unique'
    ) THEN
        -- First, remove any duplicate entries (keep the most recent)
        DELETE FROM onboarding_answers a
        WHERE id NOT IN (
            SELECT DISTINCT ON (user_id, session_id, question_key) id
            FROM onboarding_answers
            ORDER BY user_id, session_id, question_key, updated_at DESC
        );
        
        -- Create the unique index
        CREATE UNIQUE INDEX idx_onboarding_answers_unique 
        ON onboarding_answers(user_id, session_id, question_key);
    END IF;
END $$;

-- Also add a comment documenting this constraint
COMMENT ON TABLE onboarding_answers IS 'Stores wizard responses per step. Unique constraint on (user_id, session_id, question_key) for upsert operations.';
