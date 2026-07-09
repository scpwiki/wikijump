CREATE INDEX page_connection_reverse_lookup_idx
    ON page_connection(to_page_id, connection_type, from_page_id);

CREATE INDEX page_connection_missing_reverse_lookup_idx
    ON page_connection_missing(to_site_id, to_page_slug, connection_type, from_page_id);
