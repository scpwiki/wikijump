CREATE INDEX IF NOT EXISTS page_revision_tags_gin_idx
    ON page_revision USING GIN (tags);

CREATE INDEX IF NOT EXISTS page_site_latest_revision_live_idx
    ON page (site_id, latest_revision_id)
    WHERE deleted_at IS NULL;
