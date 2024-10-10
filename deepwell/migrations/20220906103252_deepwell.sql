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
    'site',
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
    last_name_change_added_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    last_renamed_at TIMESTAMP WITH TIME ZONE,
    email TEXT NOT NULL,  -- Can be empty, for instance with system accounts.
    email_is_alias BOOLEAN,
    email_verified_at TIMESTAMP WITH TIME ZONE,
    password TEXT NOT NULL,
    multi_factor_secret TEXT,
    multi_factor_recovery_codes TEXT[],
    locales TEXT[] NOT NULL,
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

    -- Locale must be unset for system users, but set for everyone else.
    CHECK ((user_type = 'system' AND locales = '{}') OR (user_type != 'system' AND locales != '{}')),

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
    layout TEXT,  -- Default page layout for the site

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
-- Relations
--

-- See docs/relation.md for more information

CREATE TYPE relation_object_type AS ENUM (
    'site',
    'user',
    'page',
    'file'
);

CREATE TABLE relation (
    relation_id BIGSERIAL PRIMARY KEY,
    relation_type TEXT NOT NULL,  -- check enum value in runtime
    dest_type relation_object_type NOT NULL,
    dest_id BIGINT NOT NULL,
    from_type relation_object_type NOT NULL,
    from_id BIGINT NOT NULL,
    metadata JSON NOT NULL DEFAULT '{}',
    created_by BIGINT NOT NULL REFERENCES "user"(user_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    overwritten_by BIGINT REFERENCES "user"(user_id),
    overwritten_at TIMESTAMP WITH TIME ZONE,
    deleted_by BIGINT REFERENCES "user"(user_id),
    deleted_at TIMESTAMP WITH TIME ZONE,

    UNIQUE (relation_type, dest_type, dest_id, from_type, from_id, overwritten_at, deleted_at),
    CHECK ((overwritten_by IS NULL) = (overwritten_at IS NULL)),  -- ensure overwritten field consistency
    CHECK ((deleted_by IS NULL) = (deleted_at IS NULL)),          -- ensure deleted field consistency
    CHECK (
        ((overwritten_by IS NULL) AND (deleted_at IS NULL)) OR    -- entries are active
        ((overwritten_by IS NULL) != (deleted_at IS NULL))        -- or they are overwritten XOR deleted
    )
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
    layout TEXT, -- category-specific override for DOM layout

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
    layout TEXT, -- page-specific override for DOM layout

    UNIQUE (site_id, slug, deleted_at)
);

--
-- Page revisions and contents
--

-- Enum types for page_revision
CREATE TYPE page_revision_type AS ENUM (
    -- standard
    'regular',
    'rollback',
    'undo',

    -- special
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
    contents TEXT NOT NULL,

    CHECK (length(hash) = 16)  -- KangarooTwelve hash size, 128 bits
);

-- Main revision table
CREATE TABLE page_revision (
    revision_id BIGSERIAL PRIMARY KEY,
    revision_type page_revision_type NOT NULL DEFAULT 'regular',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
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
    CHECK (revision_type NOT IN ('regular', 'rollback', 'undo') OR changes != '{}'),

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
    to_site_id BIGINT REFERENCES site(site_id),
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
-- Blobs
--

-- Manages blobs that are being uploaded by the user
CREATE TABLE blob_pending (
    external_id TEXT PRIMARY KEY,
    created_by BIGINT NOT NULL REFERENCES "user"(user_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    expected_length BIGINT NOT NULL CHECK (expected_length >= 0),
    s3_path TEXT NOT NULL CHECK (length(s3_path) > 1),
    s3_hash BYTEA,  -- NULL means not yet moved, NOT NULL means deleted from s3_path
    presign_url TEXT NOT NULL CHECK (length(presign_url) > 1),

    CHECK (expires_at > created_at),                 -- expiration time is not in the relative past
    CHECK (length(external_id) = 24),                -- default length for a cuid2
    CHECK (s3_hash IS NULL OR length(s3_hash) = 64)  -- SHA-512 hash size, if present
);

--
-- Files
--

-- Enum types for file_revision
CREATE TYPE file_revision_type AS ENUM (
    -- standard
    'regular',
    'rollback',

    -- special
    'create',
    'delete',
    'undelete',
    'move'
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
    site_id BIGINT NOT NULL REFERENCES site(site_id),

    UNIQUE (page_id, name, deleted_at)
);

CREATE TABLE file_revision (
    revision_id BIGSERIAL PRIMARY KEY,
    revision_type file_revision_type NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    revision_number INTEGER NOT NULL,
    file_id BIGINT NOT NULL REFERENCES file(file_id),
    page_id BIGINT NOT NULL REFERENCES page(page_id),
    site_id BIGINT NOT NULL REFERENCES site(site_id),
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

    -- Ensure array is not empty for regular revisions
    CHECK (revision_type NOT IN ('regular', 'rollback') OR changes != '{}'),

    -- Ensure page creations are always the first revision
    CHECK (revision_number != 0 OR revision_type = 'create'),

    -- For logical consistency, and adding an index
    UNIQUE (file_id, page_id, revision_number)
);

--
-- Direct Messages
--

CREATE TYPE message_recipient_type AS ENUM (
    'regular',
    'cc',
    'bcc'
);

-- A "record" is the underlying message data, with its contents, attachments,
-- and associated metadata such as sender and recipient(s).
CREATE TABLE message_record (
    external_id TEXT PRIMARY KEY, -- ID comes from message_draft
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    drafted_at TIMESTAMP WITH TIME ZONE NOT NULL,
    retracted_at TIMESTAMP WITH TIME ZONE,
    sender_id BIGINT NOT NULL REFERENCES "user"(user_id),

    -- Text contents
    subject TEXT NOT NULL,
    wikitext_hash BYTEA NOT NULL REFERENCES text(hash),
    compiled_hash BYTEA NOT NULL REFERENCES text(hash),
    compiled_at TIMESTAMP WITH TIME ZONE NOT NULL,
    compiled_generator TEXT NOT NULL,

    -- Flags
    reply_to TEXT REFERENCES message_record(external_id),
    forwarded_from TEXT REFERENCES message_record(external_id),

    CHECK (length(external_id) = 24)  -- default length for a cuid2
);

-- A "message" is a particular copy of a record.
-- If Alice sends Bob a message and CC's Charlie, then
-- there is one message_record and three message rows
-- (one for each recipient, including the sender's "Sent" folder).
CREATE TABLE message (
    internal_id BIGSERIAL PRIMARY KEY,
    record_id TEXT NOT NULL REFERENCES message_record(external_id),  -- The record this corresponds to
    user_id BIGINT NOT NULL REFERENCES "user"(user_id),  -- The user who owns the copy of this record

    -- Folders and flags
    flag_read BOOLEAN NOT NULL DEFAULT false,  -- A user-toggleable flag for the "unread" status.
    flag_inbox BOOLEAN NOT NULL,
    flag_outbox BOOLEAN NOT NULL,
    flag_self BOOLEAN NOT NULL,  -- Messages sent to oneself, as a kind of "notes to self" section.
    flag_trash BOOLEAN NOT NULL DEFAULT false,
    flag_star BOOLEAN NOT NULL DEFAULT false,

    -- User-customizable tagging
    tags TEXT[] NOT NULL DEFAULT '{}',

    UNIQUE (record_id, user_id),
    CHECK (NOT (flag_self AND flag_inbox))  -- If something is sent to oneself, it cannot be in the inbox
);

CREATE TABLE message_recipient (
    record_id TEXT NOT NULL REFERENCES message_record(external_id),
    recipient_id BIGINT NOT NULL REFERENCES "user"(user_id),
    recipient_type message_recipient_type NOT NULL,

    PRIMARY KEY (record_id, recipient_id, recipient_type)
);

CREATE TABLE message_draft (
    external_id TEXT PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    user_id BIGINT NOT NULL REFERENCES "user"(user_id),
    recipients JSON NOT NULL,

    -- Text contents
    subject TEXT NOT NULL,
    wikitext_hash BYTEA NOT NULL REFERENCES text(hash),
    compiled_hash BYTEA NOT NULL REFERENCES text(hash),
    compiled_at TIMESTAMP WITH TIME ZONE NOT NULL,
    compiled_generator TEXT NOT NULL,

    -- Flags
    reply_to TEXT REFERENCES message_record(external_id),
    forwarded_from TEXT REFERENCES message_record(external_id),

    CHECK (length(external_id) = 24)  -- default length for a cuid2
);

-- If a message has been reported, then a row for it is created here.
-- Messages can be reported per-site or globally (at the platform level).
CREATE TABLE message_report (
    message_id BIGINT NOT NULL REFERENCES message(internal_id),
    reported_to_site_id BIGINT REFERENCES site(site_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    reason TEXT NOT NULL,

    PRIMARY KEY (message_id, reported_to_site_id)
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
