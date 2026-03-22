-- Add deleted_at column to uploads table for soft delete
ALTER TABLE uploads
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS blob_id UUID,
    ADD COLUMN IF NOT EXISTS document_type VARCHAR(50),
    ADD COLUMN IF NOT EXISTS description TEXT,
    ADD COLUMN IF NOT EXISTS is_blob_stored BOOLEAN DEFAULT false,
    ADD COLUMN IF NOT EXISTS previous_version_id UUID REFERENCES uploads(id);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_uploads_deleted_at ON uploads(deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_uploads_blob ON uploads(blob_id) WHERE blob_id IS NOT NULL;
