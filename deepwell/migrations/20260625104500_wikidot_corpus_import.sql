-- Wikidot corpus mirror import support.
--
-- The import data represents the current Wikidot page snapshot held by the
-- operator-side corpus. It intentionally does not fabricate historical page
-- revisions or individual vote rows.

CREATE TABLE wikidot_corpus_import_run (
    import_run_id BIGSERIAL PRIMARY KEY,
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    source_branch TEXT NOT NULL,
    source_site TEXT NOT NULL,
    manifest_sha256 BYTEA NOT NULL CHECK (octet_length(manifest_sha256) = 32),
    manifest_row_count BIGINT NOT NULL CHECK (manifest_row_count >= 0),
    complete_inventory BOOLEAN NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('planning', 'running', 'metadata_done', 'rendering', 'done', 'failed')),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ,
    summary JSONB NOT NULL DEFAULT '{}'::JSONB
);

CREATE TABLE wikidot_page_snapshot (
    page_id BIGINT PRIMARY KEY REFERENCES page(page_id) ON DELETE CASCADE,
    source_branch TEXT NOT NULL,
    source_site TEXT NOT NULL,
    source_entity_id UUID NOT NULL,
    source_fullname TEXT NOT NULL,
    source_created_at TIMESTAMPTZ NOT NULL,
    source_updated_at TIMESTAMPTZ NOT NULL,
    source_revision_count INTEGER NOT NULL CHECK (source_revision_count >= 0),
    imported_rating BIGINT NOT NULL,
    created_by_name TEXT,
    updated_by_name TEXT,
    title_shown TEXT,
    parent_fullname TEXT,
    comments INTEGER NOT NULL CHECK (comments >= 0),
    commented_at TIMESTAMPTZ,
    commented_by_name TEXT,
    source_sha256 BYTEA NOT NULL CHECK (octet_length(source_sha256) = 32),
    meta_sha256 BYTEA NOT NULL CHECK (octet_length(meta_sha256) = 32),
    meta_json JSONB NOT NULL,
    last_import_run_id BIGINT NOT NULL REFERENCES wikidot_corpus_import_run(import_run_id),
    imported_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(source_site, source_entity_id),
    UNIQUE(source_site, source_fullname)
);

CREATE INDEX wikidot_page_snapshot_updated_idx
    ON wikidot_page_snapshot(source_site, source_updated_at);

CREATE TABLE wikidot_corpus_import_item (
    import_run_id BIGINT NOT NULL REFERENCES wikidot_corpus_import_run(import_run_id) ON DELETE CASCADE,
    source_entity_id UUID NOT NULL,
    source_fullname TEXT NOT NULL,
    page_id BIGINT REFERENCES page(page_id) ON DELETE SET NULL,
    source_sha256 BYTEA NOT NULL CHECK (octet_length(source_sha256) = 32),
    meta_sha256 BYTEA NOT NULL CHECK (octet_length(meta_sha256) = 32),
    state TEXT NOT NULL CHECK (state IN (
        'pending',
        'shell_ready',
        'snapshot_ready',
        'parent_pending',
        'render_pending',
        'render_running',
        'rendered',
        'render_failed',
        'done',
        'failed'
    )),
    attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    lease_until TIMESTAMPTZ,
    error JSONB,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY(import_run_id, source_entity_id)
);

CREATE INDEX wikidot_corpus_import_item_claim_idx
    ON wikidot_corpus_import_item(import_run_id, state, lease_until);

CREATE INDEX wikidot_corpus_import_item_page_idx
    ON wikidot_corpus_import_item(page_id);
