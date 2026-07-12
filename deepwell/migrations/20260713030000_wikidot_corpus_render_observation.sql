-- Durable, per-attempt performance observations for trusted corpus renders.
-- This table is deliberately additive: finalizer item state and lease fencing
-- remain owned by wikidot_corpus_import_item.

CREATE TABLE wikidot_corpus_render_observation (
    import_run_id BIGINT NOT NULL,
    source_entity_id UUID NOT NULL,
    pass TEXT NOT NULL CHECK (pass IN ('pass1', 'pass2')),
    attempt INTEGER NOT NULL CHECK (attempt > 0),
    page_id BIGINT REFERENCES page(page_id) ON DELETE SET NULL,
    outcome TEXT NOT NULL CHECK (outcome IN ('running', 'rendered', 'done', 'render_failed')),
    budget_us BIGINT NOT NULL DEFAULT 800000 CHECK (budget_us = 800000),
    pipeline_us BIGINT CHECK (pipeline_us >= 0),
    total_us BIGINT CHECK (total_us >= 0),
    complete BOOLEAN NOT NULL DEFAULT FALSE,
    dominant_scope TEXT,
    dominant_stage TEXT,
    terminal_scope TEXT,
    terminal_stage TEXT,
    timings JSONB NOT NULL DEFAULT '{}'::JSONB CHECK (jsonb_typeof(timings) = 'object'),
    dimensions JSONB NOT NULL DEFAULT '{}'::JSONB CHECK (jsonb_typeof(dimensions) = 'object'),
    error_fingerprint TEXT,
    post_commit_error BOOLEAN NOT NULL DEFAULT FALSE,
    observed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ,
    CHECK (
        (pass = 'pass1' AND outcome IN ('running', 'rendered', 'render_failed'))
        OR (pass = 'pass2' AND outcome IN ('running', 'done', 'render_failed'))
    ),
    CHECK ((dominant_scope IS NULL) = (dominant_stage IS NULL)),
    CHECK ((terminal_scope IS NULL) = (terminal_stage IS NULL)),
    CHECK (outcome NOT IN ('rendered', 'done') OR page_id IS NOT NULL),
    CHECK (
        (
            complete = FALSE
            AND outcome = 'running'
            AND pipeline_us IS NULL
            AND total_us IS NULL
            AND finished_at IS NULL
            AND error_fingerprint IS NULL
            AND post_commit_error = FALSE
        )
        OR (
            complete = FALSE
            AND outcome IN ('rendered', 'done')
            AND pipeline_us IS NOT NULL
            AND total_us IS NULL
            AND finished_at IS NULL
            AND error_fingerprint IS NULL
            AND post_commit_error = FALSE
        )
        OR (
            complete = TRUE
            AND outcome IN ('rendered', 'done')
            AND pipeline_us IS NOT NULL
            AND total_us IS NOT NULL
            AND total_us >= pipeline_us
            AND finished_at IS NOT NULL
            AND error_fingerprint IS NULL
            AND post_commit_error = FALSE
        )
        OR (
            complete = TRUE
            AND outcome = 'render_failed'
            AND pipeline_us IS NOT NULL
            AND total_us IS NOT NULL
            AND total_us = pipeline_us
            AND finished_at IS NOT NULL
            AND error_fingerprint IS NOT NULL
            AND error_fingerprint ~ '^[0-9a-f]{64}$'
            AND post_commit_error = FALSE
        )
    ),
    PRIMARY KEY(import_run_id, source_entity_id, pass, attempt),
    FOREIGN KEY(import_run_id, source_entity_id)
        REFERENCES wikidot_corpus_import_item(import_run_id, source_entity_id)
        ON DELETE CASCADE
);

CREATE INDEX wikidot_corpus_render_observation_inventory_idx
    ON wikidot_corpus_render_observation(import_run_id, pass, complete, total_us DESC);

CREATE INDEX wikidot_corpus_render_observation_stage_idx
    ON wikidot_corpus_render_observation(import_run_id, pass, dominant_stage);
