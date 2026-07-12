-- Support ListPages created_by lookups without fabricating Wikidot user IDs.
CREATE INDEX wikidot_page_snapshot_created_by_name_normalized_idx
    ON wikidot_page_snapshot (
        (replace(replace(lower(btrim(created_by_name)), '_', '-'), ' ', '-')),
        page_id
    )
    WHERE created_by_name IS NOT NULL;
