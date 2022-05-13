-- Temporary ad hoc migration to add what we need for tests to take place.
-- This will be removed when we are using native DEEPWELL migrations
-- and can just run it as part of preparation then.
--
-- See https://scuttle.atlassian.net/browse/WJ-1026

-- Utilties

CREATE OR REPLACE FUNCTION json_array_to_text_array(_js json)
    RETURNS TEXT[]
    LANGUAGE SQL
    IMMUTABLE
    PARALLEL
    SAFE
AS
    'SELECT array(SELECT json_array_elements_text(_js))';

-- Types

CREATE TYPE revision_type AS ENUM (
    'regular',
    'create',
    'delete',
    'undelete'
);

CREATE TYPE revision_change AS ENUM (
    'wikitext',
    'title',
    'alt_title',
    'slug',
    'tags'
)

-- Tables

CREATE TABLE page_category (
    category_id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    slug TEXT NOT NULL,

    UNIQUE (site_id, slug)
);

CREATE TABLE page (
    page_id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE,
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    page_category_id BIGINT NOT NULL REFERENCES page_category(category_id),
    slug TEXT NOT NULL,
    discussion_thread_id BIGINT REFERENCES forum_thread(thread_id),

    UNIQUE (site_id, slug, deleted_at)
);

CREATE TABLE page_revision (
    revision_id BIGSERIAL PRIMARY KEY,
    revision_type revision_type NOT NULL DEFAULT 'regular',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    revision_number INT NOT NULL,
    page_id BIGINT NOT NULL REFERENCES page(page_id),
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    user_id BIGINT NOT NULL REFERENCES users(id),
    changes JSON NOT NULL, -- List of changes in this revision
    wikitext_hash BYTEA NOT NULL REFERENCES text(hash),
    compiled_hash BYTEA NOT NULL REFERENCES text(hash),
    compiled_at TIMESTAMP WITH TIME ZONE NOT NULL,
    compiled_generator TEXT NOT NULL,
    comments TEXT NOT NULL,
    hidden JSON NOT NULL DEFAULT '[]', -- List of fields to be hidden/suppressed
    title TEXT NOT NULL,
    alt_title TEXT,
    slug TEXT NOT NULL,
    tags JSON NOT NULL DEFAULT '[]', -- Should be sorted and deduplicated before insertion

    -- NOTE: json_array_to_text_array() is needed while we're still on JSON

    -- Ensure array only contains valid values
    -- Change this to use the 'revision_change' type later
    CHECK (json_array_to_text_array(changes) <@ '{
        \"wikitext\",
        \"title\",
        \"alt_title\",
        \"slug\",
        \"tags\"
    }'),

    -- Ensure first revision reports all changes
    --
    -- This is implemented  by seeing if it's a superset or equal to all valid values.
    -- Since we already check if it's a subset or equal, this is the same as
    -- strict equivalence, but without regard for ordering.
    CHECK (
        revision_type != 'create' OR
        json_array_to_text_array(changes) @> '{
            \"wikitext\",
            \"title\",
            \"alt_title\",
            \"slug\",
            \"tags\"
        }'
    ),

    -- Ensure array is not empty for regular revisions
    CHECK (revision_type != 'regular' OR json_array_to_text_array(changes) != '{}'),

    -- Ensure page creations are always the first revision
    CHECK (revision_number != 0 OR revision_type = 'create'),

    -- For logical consistency, and adding an index
    UNIQUE (page_id, site_id, revision_number)
);

CREATE TABLE page_parent (
    parent_page_id BIGINT REFERENCES page(page_id),
    child_page_id BIGINT REFERENCES page(page_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),

    PRIMARY KEY (parent_page_id, child_page_id)
);

CREATE TABLE page_attribution (
    page_id BIGINT REFERENCES page(page_id),
    user_id BIGINT REFERENCES users(id),
    -- Text enum describing the kind of attribution
    -- Currently synced to Crom: 'author', 'rewrite', 'translator', 'maintainer'
    attribution_type TEXT NOT NULL,
    attribution_date DATE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),

    PRIMARY KEY (page_id, user_id, attribution_type, attribution_date)
);

CREATE TABLE page_lock (
    page_lock_id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE,
    -- Text enum describing what kind of lock (e.g. authors only, staff only)
    -- Currently the only value is 'wikidot' (meaning mods+ only)
    lock_type TEXT NOT NULL,
    page_id BIGINT NOT NULL REFERENCES page(page_id),
    user_id BIGINT NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,

    UNIQUE (page_id, deleted_at)
);

CREATE TABLE page_link (
    page_id BIGINT REFERENCES page(page_id),
    url TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE,
    count INT NOT NULL CHECK (count > 0),

    PRIMARY KEY (page_id, url)
);

CREATE TABLE page_connection (
    from_page_id BIGINT REFERENCES page(page_id),
    to_page_id BIGINT REFERENCES page(page_id),
    connection_type TEXT
        CHECK (connection_type = ANY(ARRAY[
            'include-messy',
            'include-elements',
            'component',
            'link',
            'redirect'
        ])),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE,
    count INT NOT NULL CHECK (count > 0),

    PRIMARY KEY (from_page_id, to_page_id, connection_type)
);


CREATE TABLE page_connection_missing (
    from_page_id BIGINT REFERENCES page(page_id),
    to_site_id BIGINT REFERENCES page(page_id),
    to_page_slug TEXT,
    connection_type TEXT
        CHECK (connection_type = ANY(ARRAY[
            'include-messy',
            'include-elements',
            'component',
            'link',
            'redirect'
        ])),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE,
    count INT NOT NULL CHECK (count > 0),

    PRIMARY KEY (from_page_id, to_site_id, to_page_slug, connection_type)
);

-- No unique constraint because that creates a separate index,
-- which will impact performance. Instead we add a CHECK constraint.
CREATE TABLE text (
    hash BYTEA PRIMARY KEY,
    contents TEXT COMPRESSION pglz NOT NULL,

    CHECK (hash = digest(contents, 'sha512'))
);
