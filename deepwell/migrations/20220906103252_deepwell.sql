-- Add main DEEPWELL tables
--
-- This revision will be continually amended until the bulk / foundation of tables are present.
-- This is to ease development, and after things are stable and "production" starts to exist,
-- further database migrations will be regular migration files.

--
-- User
--

CREATE TYPE user_type AS ENUM (
    'regular',
    'system',
    'bot'
);

CREATE TABLE "user" (
    user_id BIGSERIAL PRIMARY KEY,
    user_type user_type NOT NULL DEFAULT 'regular',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE,
    from_wikidot BOOLEAN NOT NULL DEFAULT false,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    name_changes_left SMALLINT NOT NULL,  -- Default set in runtime configuration.
    last_renamed_at TIMESTAMP WITH TIME ZONE,
    email TEXT NOT NULL,
    email_is_alias BOOLEAN,
    email_verified_at TIMESTAMP WITH TIME ZONE,
    password TEXT NOT NULL,
    multi_factor_secret TEXT,
    multi_factor_recovery_codes TEXT[],
    locale TEXT NOT NULL,
    avatar_s3_hash BYTEA,
    real_name TEXT,
    gender TEXT,
    birthday DATE,
    location TEXT,
    biography TEXT,
    user_page TEXT,

    -- Name uniqueness constraints
    UNIQUE (name, deleted_at),
    UNIQUE (slug, deleted_at),

    -- Both MFA columns should either be set or unset
    CHECK ((multi_factor_secret IS NULL) = (multi_factor_recovery_codes IS NULL)),

    -- Strings should either be NULL or non-empty (and within limits)
    CHECK (real_name IS NULL OR (length(real_name) > 0 AND length(real_name) < 300)),
    CHECK (gender IS NULL OR (length(gender) > 0 AND length(gender) < 100)),
    CHECK (location IS NULL OR (length(location) > 0 AND length(location) < 100)),
    CHECK (biography IS NULL OR (length(biography) > 0 AND length(biography) < 4000)),
    CHECK (user_page IS NULL OR (length(user_page) > 0 AND length(user_page) < 100)),

    CHECK (name_changes_left >= 0),                                 -- Value cannot be negative
    CHECK (avatar_s3_hash IS NULL OR length(avatar_s3_hash) = 64)   -- SHA-512 hash size (if set)
);

CREATE TABLE user_bot_owner (
    bot_user_id BIGINT REFERENCES "user"(user_id),
    human_user_id BIGINT REFERENCES "user"(user_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    description TEXT NOT NULL,

    PRIMARY KEY (bot_user_id, human_user_id)
);

--
-- Site
--

CREATE TABLE site (
    site_id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE,
    from_wikidot BOOLEAN NOT NULL DEFAULT false,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    tagline TEXT NOT NULL,
    description TEXT NOT NULL,
    locale TEXT NOT NULL,
    default_page TEXT NOT NULL DEFAULT 'start',
    custom_domain TEXT,  -- Dependency cycle, add foreign key constraint after

    UNIQUE (slug, deleted_at)
);

CREATE TABLE site_domain (
    domain TEXT PRIMARY KEY,
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),

    CHECK (length(domain) > 0)
);

ALTER TABLE site
    ADD CONSTRAINT site_custom_domain_fk
    FOREIGN KEY (custom_domain) REFERENCES site_domain(domain);

--
-- Site Membership
--

CREATE TABLE site_member (
    membership_id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES "user"(user_id),
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    deleted_at TIMESTAMP WITH TIME ZONE,

    UNIQUE (user_id, site_id, deleted_at)
);

--
-- Aliases
--

CREATE TYPE alias_type AS ENUM (
    'site',
    'user'
);

CREATE TABLE alias (
    alias_id BIGSERIAL PRIMARY KEY,
    alias_type alias_type NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    created_by BIGINT NOT NULL REFERENCES "user"(user_id),
    target_id BIGINT NOT NULL,
    slug TEXT NOT NULL,

    UNIQUE (alias_type, slug)
);

--
-- Interactions
--

-- See also https://github.com/scpwiki/wikijump/blob/legacy-php/web/database/migrations/2021_07_30_231009_create_interactions_table.php
-- and https://github.com/scpwiki/wikijump/blob/legacy-php/web/app/Models/Interaction.php

CREATE TYPE interaction_object_type AS ENUM (
    'site',
    'user',
    'page',
    'file'
);

CREATE TYPE interaction_type AS ENUM (
    'member',
    'block',
    'watch',
    'star',
);

CREATE TABLE interaction (
    source_type interaction_object_type NOT NULL,
    source_id BIGINT NOT NULL,
    interaction_type interaction_type NOT NULL,
    target_type interaction_object_type NOT NULL,
    target_id BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),

    PRIMARY KEY (source_type, source_id, interaction_type, target_type, target_id)
);

--
-- Session
--

CREATE TABLE session (
    session_token TEXT PRIMARY KEY CHECK (length(session_token) > 48),
    user_id BIGINT NOT NULL REFERENCES "user"(user_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL CHECK (expires_at > created_at),
    ip_address TEXT NOT NULL,  -- TODO change to INET
    user_agent TEXT NOT NULL,
    restricted BOOLEAN NOT NULL
);

--
-- Page
--

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
    from_wikidot BOOLEAN NOT NULL DEFAULT false,
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    latest_revision_id BIGINT, -- nullable to avoid an initial page_revision dependency cycle
    page_category_id BIGINT NOT NULL REFERENCES page_category(category_id),
    slug TEXT NOT NULL,
    discussion_thread_id BIGINT, -- TODO: add REFERENCES to forum threads

    UNIQUE (site_id, slug, deleted_at)
);

--
-- Page revisions and contents
--

-- Enum types for page_revision
CREATE TYPE page_revision_type AS ENUM (
    'regular',
    'create',
    'delete',
    'undelete',
    'move'
);

CREATE TYPE page_revision_change AS ENUM (
    'wikitext',
    'title',
    'alt_title',
    'slug',
    'tags'
);

-- No unique constraint for 'contents' because that would create
-- create a separate index, which will impact performance.
--
-- If the KangarooTwelve hash algorithm was available in pgcrypto
-- we'd check directly (hash = digest(contents, 'kangarootwelve')),
-- but since we can't we'll just verify the hash length.
CREATE TABLE text (
    hash BYTEA PRIMARY KEY,
    contents TEXT COMPRESSION pglz NOT NULL,

    CHECK (length(hash) = 16)  -- KangarooTwelve hash size, 128 bits
);

-- Main revision table
CREATE TABLE page_revision (
    revision_id BIGSERIAL PRIMARY KEY,
    revision_type page_revision_type NOT NULL DEFAULT 'regular',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    revision_number INT NOT NULL,
    page_id BIGINT NOT NULL REFERENCES page(page_id),
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    user_id BIGINT NOT NULL REFERENCES "user"(user_id),
    from_wikidot BOOLEAN NOT NULL DEFAULT false,
    changes TEXT[] NOT NULL, -- List of changes in this revision
    wikitext_hash BYTEA NOT NULL REFERENCES text(hash),
    compiled_hash BYTEA NOT NULL REFERENCES text(hash),
    compiled_at TIMESTAMP WITH TIME ZONE NOT NULL,
    compiled_generator TEXT NOT NULL,
    comments TEXT NOT NULL,
    hidden TEXT[] NOT NULL DEFAULT '{}', -- List of fields to be hidden/suppressed
    title TEXT NOT NULL,
    alt_title TEXT,
    slug TEXT NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{}', -- Should be sorted and deduplicated before insertion

    -- Ensure array only contains valid values
    -- Change this to use the 'page_revision_change' type later
    CHECK (changes <@ '{
        wikitext,
        title,
        alt_title,
        slug,
        tags
    }'),

    -- Ensure first revision reports all changes
    --
    -- This is implemented  by seeing if it's a superset or equal to all valid values.
    -- Since we already check if it's a subset or equal, this is the same as
    -- strict equivalence, but without regard for ordering.
    CHECK (
        revision_type != 'create' OR
        changes @> '{
            wikitext,
            title,
            alt_title,
            slug,
            tags
        }'
    ),

    -- Ensure array is not empty for regular revisions
    CHECK (revision_type != 'regular' OR changes != '{}'),

    -- Ensure page creations are always the first revision
    CHECK (revision_number != 0 OR revision_type = 'create'),

    -- For logical consistency, and adding an index
    UNIQUE (page_id, site_id, revision_number)
);

-- Add foreign key constraint for latest_revision_id
ALTER TABLE page ADD CONSTRAINT page_revision_revision_id_fk
    FOREIGN KEY (latest_revision_id) REFERENCES page_revision(revision_id);

--
-- Page metadata
--

CREATE TABLE page_parent (
    parent_page_id BIGINT REFERENCES page(page_id),
    child_page_id BIGINT REFERENCES page(page_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),

    PRIMARY KEY (parent_page_id, child_page_id)
);

CREATE TABLE page_attribution (
    page_id BIGINT REFERENCES page(page_id),
    user_id BIGINT REFERENCES "user"(user_id),
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
    expires_at TIMESTAMP WITH TIME ZONE,
    from_wikidot BOOLEAN NOT NULL DEFAULT false,
    -- Text enum describing what kind of lock (e.g. authors only, staff only)
    -- Currently the only value is 'wikidot' (meaning mods+ only)
    lock_type TEXT NOT NULL,
    page_id BIGINT NOT NULL REFERENCES page(page_id),
    user_id BIGINT NOT NULL REFERENCES "user"(user_id),
    reason TEXT NOT NULL,

    UNIQUE (page_id, deleted_at)
);

--
-- Page backlinks tracking
--

-- Enum types for page backlinks
CREATE TYPE page_connection_type AS ENUM (
    'include-messy',
    'include-elements',
    'component',
    'link',
    'redirect'
);

CREATE TABLE page_link (
    page_id BIGINT REFERENCES page(page_id),
    url TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    count INT NOT NULL CHECK (count > 0),

    PRIMARY KEY (page_id, url)
);

CREATE TABLE page_connection (
    from_page_id BIGINT REFERENCES page(page_id),
    to_page_id BIGINT REFERENCES page(page_id),
    connection_type TEXT, -- Cannot use page_connection_type right now because Sea-ORM issues
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    count INT NOT NULL CHECK (count > 0),

    PRIMARY KEY (from_page_id, to_page_id, connection_type)
);

CREATE TABLE page_connection_missing (
    from_page_id BIGINT REFERENCES page(page_id),
    to_site_id BIGINT REFERENCES page(page_id),
    to_page_slug TEXT,
    connection_type TEXT, -- Ditto
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    count INT NOT NULL CHECK (count > 0),

    PRIMARY KEY (from_page_id, to_site_id, to_page_slug, connection_type)
);

--
-- Page votes
--

CREATE TABLE page_vote (
    page_vote_id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    disabled_at TIMESTAMP WITH TIME ZONE,
    disabled_by BIGINT REFERENCES "user"(user_id),
    from_wikidot BOOLEAN NOT NULL DEFAULT false,
    page_id BIGINT NOT NULL REFERENCES page(page_id),
    user_id BIGINT NOT NULL REFERENCES "user"(user_id),
    value SMALLINT NOT NULL,

    UNIQUE (page_id, user_id, deleted_at),
    CHECK ((disabled_at IS NULL) = (disabled_by IS NULL))
);

--
-- Files
--

-- Enum types for file_revision
CREATE TYPE file_revision_type AS ENUM (
    'create',
    'update',
    'delete',
    'undelete'
);

CREATE TYPE file_revision_change AS ENUM (
    'name',
    'blob',
    'mime',
    'licensing'
);

CREATE TABLE file (
    file_id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE,
    from_wikidot BOOLEAN NOT NULL DEFAULT false,
    name TEXT NOT NULL,
    page_id BIGINT NOT NULL REFERENCES page(page_id),

    UNIQUE (page_id, name, deleted_at)
);

CREATE TABLE file_revision (
    revision_id BIGSERIAL PRIMARY KEY,
    revision_type file_revision_type NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    revision_number INTEGER NOT NULL,
    file_id BIGINT NOT NULL REFERENCES file(file_id),
    page_id BIGINT NOT NULL REFERENCES page(page_id),
    user_id BIGINT NOT NULL REFERENCES "user"(user_id),
    name TEXT NOT NULL,
    s3_hash BYTEA NOT NULL,
    mime_hint TEXT NOT NULL,
    size_hint BIGINT NOT NULL,
    licensing JSON NOT NULL,
    changes TEXT[] NOT NULL DEFAULT '{}', -- List of changes in this revision
    comments TEXT NOT NULL,
    hidden TEXT[] NOT NULL DEFAULT '{}', -- List of fields to be hidden/suppressed

    CHECK (length(name) > 0 AND length(name) < 256),  -- Constrain filename length
    CHECK (length(s3_hash) = 64),                     -- SHA-512 hash size
    CHECK (mime_hint != ''),                          -- Should have a MIME hint

    -- Ensure array only contains valid values
    -- Change this to use the 'page_revision_change' type later
    CHECK (changes <@ '{
        page,
        name,
        blob,
        mime,
        licensing
    }'),

    -- Ensure first revision reports all changes
    --
    -- This is implemented  by seeing if it's a superset or equal to all valid values.
    -- Since we already check if it's a subset or equal, this is the same as
    -- strict equivalence, but without regard for ordering.
    CHECK (
        revision_type != 'create' OR
        changes @> '{
            page,
            name,
            blob,
            mime,
            licensing
        }'
    ),

    -- Ensure array is not empty for update revisions
    CHECK (revision_type != 'update' OR changes != '{}'),

    -- Ensure page creations are always the first revision
    CHECK (revision_number != 0 OR revision_type = 'create'),

    -- For logical consistency, and adding an index
    UNIQUE (file_id, page_id, revision_number)
);

--
-- Filters
--

-- Refers both to system and site filters.
--
-- If site_id is NULL, then it is a system (platform-wide) filter. It affects all sites.
-- If site_id is set, then it is a site filter, affecting only that site.
--
-- If a filter has all the "affects_*" columns false, then it is effectively disabled.
CREATE TABLE filter (
    filter_id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE,
    site_id BIGINT REFERENCES site(site_id),
    affects_user BOOLEAN NOT NULL DEFAULT false,
    affects_email BOOLEAN NOT NULL DEFAULT false,
    affects_page BOOLEAN NOT NULL DEFAULT false,
    affects_file BOOLEAN NOT NULL DEFAULT false,
    affects_forum BOOLEAN NOT NULL DEFAULT false,
    regex TEXT NOT NULL,
    description TEXT NOT NULL,

    UNIQUE (site_id, regex, deleted_at)
);
