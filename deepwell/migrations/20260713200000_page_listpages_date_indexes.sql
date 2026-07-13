CREATE INDEX page_site_created_at_live_idx
    ON page (site_id, created_at, page_id)
    WHERE deleted_at IS NULL;

CREATE INDEX page_site_updated_at_live_idx
    ON page (site_id, updated_at, page_id)
    WHERE deleted_at IS NULL;
