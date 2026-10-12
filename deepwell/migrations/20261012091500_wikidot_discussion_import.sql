-- Public source identity observations do not create platform accounts, roles,
-- sessions, or a fabricated registration date. Full imported profiles remain
-- the owner of profile data; forum display may use a newer observed identity.
CREATE TABLE wikidot_identity_observation (
    user_id BIGINT NOT NULL REFERENCES known_user(user_id),
    source_sha256 BYTEA NOT NULL CHECK (octet_length(source_sha256) = 32),
    observed_at TIMESTAMPTZ NOT NULL,
    name TEXT NOT NULL CHECK (name <> ''),
    slug TEXT NOT NULL CHECK (slug <> ''),
    source_reference TEXT NOT NULL,
    PRIMARY KEY (user_id, source_sha256)
);

CREATE VIEW wikidot_display_user AS
WITH latest AS (
    SELECT DISTINCT ON (user_id) user_id, name, slug, observed_at
    FROM wikidot_identity_observation
    ORDER BY user_id, observed_at DESC, source_sha256 DESC
)
SELECT u.user_id::BIGINT AS user_id,
       CASE WHEN i.observed_at > u.fetched_at AND NOT u.is_deleted THEN i.name ELSE u.name END AS name,
       CASE WHEN i.observed_at > u.fetched_at AND NOT u.is_deleted THEN i.slug ELSE u.slug END AS slug,
       u.is_deleted
FROM wikidot_user u LEFT JOIN latest i ON i.user_id = u.user_id
UNION ALL
SELECT i.user_id, i.name, i.slug, FALSE AS is_deleted
FROM latest i WHERE NOT EXISTS (SELECT 1 FROM wikidot_user u WHERE u.user_id = i.user_id);

-- Each completed import retains the observed graph and its provenance. Post
-- source changes create ordinary post revisions rather than replacing them.
CREATE TABLE wikidot_discussion_import_run (
    import_run_id BIGSERIAL PRIMARY KEY,
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    page_id BIGINT NOT NULL REFERENCES page(page_id),
    forum_thread_id BIGINT NOT NULL REFERENCES forum_thread(forum_thread_id),
    importing_user_id BIGINT NOT NULL REFERENCES known_user(user_id),
    source_sha256 BYTEA NOT NULL CHECK (octet_length(source_sha256) = 32),
    manifest_sha256 BYTEA NOT NULL CHECK (octet_length(manifest_sha256) = 32),
    observed_at TIMESTAMPTZ NOT NULL,
    source_reference TEXT NOT NULL,
    manifest JSONB NOT NULL,
    imported_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (page_id, manifest_sha256)
);

CREATE TABLE wikidot_discussion_snapshot (
    page_id BIGINT PRIMARY KEY REFERENCES page(page_id),
    forum_thread_id BIGINT NOT NULL UNIQUE REFERENCES forum_thread(forum_thread_id),
    source_site TEXT NOT NULL,
    source_system_creator BOOLEAN NOT NULL,
    source_post_count INTEGER NOT NULL CHECK (source_post_count >= 0),
    last_import_run_id BIGINT NOT NULL REFERENCES wikidot_discussion_import_run(import_run_id)
);

-- A current imported revision may be the first locally stored snapshot of an
-- already edited source post. Do not invent missing historical revisions.
CREATE TABLE wikidot_discussion_post_snapshot (
    forum_post_id BIGINT PRIMARY KEY REFERENCES forum_post(forum_post_id),
    revision_id BIGINT NOT NULL REFERENCES forum_post_revision(forum_post_revision_id),
    source_has_edit BOOLEAN NOT NULL,
    render_identity_sha256 BYTEA NOT NULL CHECK (octet_length(render_identity_sha256) = 32)
);
