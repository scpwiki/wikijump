-- Saved-page output rendered ahead of a renderer generation change.
--
-- A renderer epoch or FTML bump makes every saved page stale, and the view
-- path fails closed for stale output. The next Deepwell build fills this table
-- while the current build keeps serving from page_revision, and the next build
-- moves valid rows into page_revision when it starts, so the switch needs no
-- rerender window. A row is valid only while its revision is still the page's
-- latest revision with the same source and has not been rerendered since the
-- row's inputs were read.
--
-- The statements are idempotent because install/standing/render_ahead.py runs
-- this file before the new build is activated, without recording it in
-- _sqlx_migrations, so the serving build's own migration check still passes.

CREATE TABLE IF NOT EXISTS page_revision_render_ahead (
    revision_id BIGINT PRIMARY KEY
        REFERENCES page_revision (revision_id) ON DELETE CASCADE,
    site_id BIGINT NOT NULL,
    page_id BIGINT NOT NULL,
    wikitext_hash BYTEA NOT NULL,
    source_updated_at TIMESTAMPTZ,
    compiled_body_html_hash BYTEA NOT NULL REFERENCES text (hash),
    compiled_body_styles_hash BYTEA NOT NULL REFERENCES text (hash),
    compiled_top_bar_html_hash BYTEA REFERENCES text (hash),
    compiled_side_bar_html_hash BYTEA REFERENCES text (hash),
    compiled_at TIMESTAMPTZ NOT NULL,
    compiled_generator TEXT NOT NULL,
    rendered_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS page_revision_render_ahead_generator_idx
    ON page_revision_render_ahead (compiled_generator);
