ALTER TABLE text_block
    ADD COLUMN IF NOT EXISTS s3_filename TEXT;

UPDATE text_block
SET s3_filename = page_id || '_' || block_type || '_' || block_index
WHERE s3_filename IS NULL;

ALTER TABLE text_block
    ALTER COLUMN s3_filename SET NOT NULL;

DO $$
BEGIN
    ALTER TABLE text_block
        ADD CONSTRAINT text_block_s3_filename_unique UNIQUE (s3_filename);
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;
