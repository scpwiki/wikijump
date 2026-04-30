-- Add main DEEPWELL tables
--
-- This revision will be continually amended until the bulk / foundation of tables are present.
-- This is to ease development, and after things are stable and "production" starts to exist,
-- further database migrations will be regular migration files.

--
-- User
--

-- Contains all known user IDs.
-- Used for foreign key constraints.

CREATE TABLE known_user (
    user_id BIGSERIAL PRIMARY KEY
);

-- Represents an active user on the platform.
-- See wikidot_user for interaction notes.

CREATE TABLE "user" (
    user_id BIGINT PRIMARY KEY REFERENCES known_user(user_id),
    -- Rust enum: UserType
    user_type TEXT NOT NULL DEFAULT 'regular',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    name_changes_left SMALLINT NOT NULL,  -- Default set in runtime configuration.
    last_name_change_added_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    last_renamed_at TIMESTAMP WITH TIME ZONE,
    email TEXT NOT NULL,  -- Can be empty, for instance with system accounts.
    email_verified_at TIMESTAMP WITH TIME ZONE,
    email_validation_info JSON,  -- Can be NULL if validation was skipped
    email_validation_at TIMESTAMP WITH TIME ZONE,
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
    website TEXT,
    user_page TEXT,

    -- Name uniqueness constraints
    UNIQUE (name, deleted_at),
    UNIQUE (slug, deleted_at),

    -- Both MFA columns should either be set or unset
    CHECK ((multi_factor_secret IS NULL) = (multi_factor_recovery_codes IS NULL)),

    -- Both email validation columns should either be set or unset
    CHECK ((email_validation_info IS NULL) = (email_validation_at IS NULL)),

    -- Locale must be unset for system users, but set for everyone else.
    CHECK ((user_type = 'system' AND locales = '{}') OR (user_type != 'system' AND locales != '{}')),

    -- Enum value must not be empty
    CHECK (length(user_type) > 0),

    -- Strings should either be NULL or non-empty (and within limits)
    CHECK (real_name IS NULL OR (length(real_name) > 0 AND length(real_name) < 300)),
    CHECK (gender IS NULL OR (length(gender) > 0 AND length(gender) < 100)),
    CHECK (location IS NULL OR (length(location) > 0 AND length(location) < 100)),
    CHECK (biography IS NULL OR (length(biography) > 0 AND length(biography) < 4000)),
    CHECK (website IS NULL OR (length(website) > 0 AND length(website) < 100)),
    CHECK (user_page IS NULL OR (length(user_page) > 0 AND length(user_page) < 100)),

    CHECK (name_changes_left >= 0),                                 -- Value cannot be negative
    CHECK (avatar_s3_hash IS NULL OR length(avatar_s3_hash) = 64)   -- SHA-512 hash size (if set)
);

-- Represents legacy users imported from Wikidot.
--
-- This separate table enables us to satisfy the looser constraints
-- Wikidot has on its data without compromising them for regular users.
--
-- The relationship between the user table and this one is:
-- * A row in user only indicates a Wikijump-only user account.
-- * A row in wikidot_user only indicates a Wikidot-only user account.
-- * A row in both (with the same user ID) indicates a Wikidot user who has imported their account into Wikijump.
-- * The wikidot_user table represents the static state of the Wikidot user at the time of "fetched_at". If a user has been imported to Wikijump, then any field updates are not reflected here.
-- * If a user is queried, and it does not exist in user or has an alias, then wikidot_user is queried.

CREATE TABLE wikidot_user (
    user_id INTEGER PRIMARY KEY REFERENCES known_user(user_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    fetched_at TIMESTAMP WITH TIME ZONE NOT NULL,
    is_deleted BOOLEAN NOT NULL,
    name TEXT UNIQUE CHECK (length(name) > 0),
    slug TEXT UNIQUE CHECK (length(slug) > 0),
    -- Biographical fields (optional)
    real_name TEXT,
    gender TEXT,
    birthday DATE,
    location TEXT,
    biography TEXT,
    website TEXT,
    karma SMALLINT NOT NULL CHECK (0 <= karma AND karma <= 5),
    is_pro BOOLEAN NOT NULL,

    -- Basic sanity check on fetched_at
    CHECK (created_at < fetched_at),
    -- Only deleted users can be missing a username or slug
    CHECK (is_deleted OR (name IS NOT NULL AND slug IS NOT NULL))
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
    default_page TEXT NOT NULL DEFAULT 'start' CHECK (default_page != ''),
    -- For nav pages, an empty string means that this navigation panel is disabled.
    -- A value of NULL means that the nav page is inherited from site settings,
    -- which is here. So this value must be non-null.
    top_bar_page TEXT NOT NULL DEFAULT 'nav:top',
    side_bar_page TEXT NOT NULL DEFAULT 'nav:side',

    -- Dependency cycle, add foreign key constraint after.
    --
    -- This field describes what the preferred domain is for this site.
    -- All sites have one preferred domain, and so a value of NULL is
    -- also meaningful here.
    --
    -- Say we have a site with the slug 'foo' and the main domain is 'wikijump.dev'.
    -- Therefore, the canonical domain for this site is 'foo.wikijump.dev'.
    --
    -- What is the preferred domain? It depends on the value of this column.
    -- * NULL          - This means the canonical domain is also the preferred domain.
    -- * 'example.com' - This means that the custom domain 'example.com' is preferred.
    --
    -- This value should NEVER have a main domain component. It must match a corresponding
    -- row in the site_domain (custom domains) table.
    --
    -- Observe that a site may have many custom domains, and this is unrelated to what
    -- its preferred domain is. Of course, if the preferred_domain column is not NULL,
    -- then it must be one of these site domains, it cannot belong to another site.
    preferred_domain TEXT,
    layout TEXT,                -- Default page layout for the site

    -- Rust enum: License
    license TEXT NOT NULL,      -- Default content license for the site

    -- Special condition
    -- The preferred site for the special 'www' site (the main page) must always be the
    -- canonical domain. That is, if the main domain is "wikijump.com", then the
    -- preferred site is "wikijump.com" (since the "www" is elided as a special case).
    CHECK (slug != 'www' OR preferred_domain IS NULL),

    -- Enum value must not be empty
    CHECK (length(license) > 0),

    -- Enforce site slug uniqueness
    UNIQUE (slug, deleted_at)
);

CREATE TABLE site_domain (
    domain TEXT PRIMARY KEY,
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    www_redirect BOOLEAN NOT NULL,

    CHECK (length(domain) > 0)
);

ALTER TABLE site
    ADD CONSTRAINT site_preferred_domain_fk
    FOREIGN KEY (preferred_domain) REFERENCES site_domain(domain);

--
-- Aliases
--

CREATE TABLE alias (
    alias_id BIGSERIAL PRIMARY KEY,
    -- Rust enum: AliasType
    alias_type TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    created_by BIGINT NOT NULL REFERENCES known_user(user_id),
    target_id BIGINT NOT NULL,
    slug TEXT NOT NULL,

    UNIQUE (alias_type, slug),

    -- Enum value must not be empty
    CHECK (length(alias_type) > 0)
);

--
-- Relations
--

-- See docs/relation.md for more information

CREATE TABLE relation (
    relation_id BIGSERIAL PRIMARY KEY,

    -- Rust enum: RelationType
    relation_type TEXT NOT NULL,

    -- Rust enum: RelationObjectType
    dest_type TEXT NOT NULL,
    dest_id BIGINT NOT NULL,

    -- Rust enum: RelationObjectType
    from_type TEXT NOT NULL,
    from_id BIGINT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by BIGINT NOT NULL REFERENCES known_user(user_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    overwritten_by BIGINT REFERENCES known_user(user_id),
    overwritten_at TIMESTAMP WITH TIME ZONE,
    deleted_by BIGINT REFERENCES known_user(user_id),
    deleted_at TIMESTAMP WITH TIME ZONE,

    CHECK ((overwritten_by IS NULL) = (overwritten_at IS NULL)),  -- ensure overwritten field consistency
    CHECK ((deleted_by IS NULL) = (deleted_at IS NULL)),          -- ensure deleted field consistency
    CHECK (
        ((overwritten_by IS NULL) AND (deleted_at IS NULL)) OR    -- entries are active
        ((overwritten_by IS NULL) != (deleted_at IS NULL))        -- or they are overwritten XOR deleted
    ),

    -- Enum values must not be empty
    CHECK (length(relation_type) > 0),
    CHECK (length(dest_type) > 0),
    CHECK (length(from_type) > 0)
);

CREATE UNIQUE INDEX relation_unique_general_active
    ON relation (relation_type, dest_type, dest_id, from_type, from_id, overwritten_at, deleted_at)
    WHERE relation_type <> 'page-attribution';

CREATE UNIQUE INDEX relation_unique_page_attribution_active
    ON relation (
        relation_type,
        dest_type,
        dest_id,
        from_type,
        from_id,
        (metadata ->> 'attribution_type'),
        (metadata ->> 'attribution_date'),
        coalesce(overwritten_at, 'infinity'),
        coalesce(deleted_at, 'infinity')
    )
    WHERE relation_type = 'page-attribution';

--
-- Session
--

CREATE TABLE session (
    session_token TEXT PRIMARY KEY CHECK (length(session_token) > 48),
    user_id BIGINT NOT NULL REFERENCES known_user(user_id),
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
    -- Category-specific overrides
    layout TEXT,
    -- If NULL, then inherit nav settings from the site table.
    -- Any other value is an override.
    -- Like with the site table, empty strings mean that this
    -- navigation panel is disabled.
    top_bar_page TEXT,
    side_bar_page TEXT,

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
    discussion_thread_id BIGINT, -- FK added after forum_thread is declared
    layout TEXT, -- page-specific override for DOM layout

    UNIQUE (site_id, slug, deleted_at)
);

--
-- Page revisions and contents
--

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
    revision_type TEXT NOT NULL DEFAULT 'regular',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    revision_number INT NOT NULL,
    page_id BIGINT NOT NULL REFERENCES page(page_id),
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    user_id BIGINT NOT NULL REFERENCES known_user(user_id),
    from_wikidot BOOLEAN NOT NULL DEFAULT false,
    changes TEXT[] NOT NULL, -- List of changes in this revision
    wikitext_hash BYTEA NOT NULL REFERENCES text(hash),
    compiled_body_html_hash BYTEA NOT NULL REFERENCES text(hash),
    compiled_top_bar_html_hash BYTEA REFERENCES text(hash),
    compiled_side_bar_html_hash BYTEA REFERENCES text(hash),
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

    -- Enum value must not be empty
    CHECK (length(revision_type) > 0),

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
    user_id BIGINT NOT NULL REFERENCES known_user(user_id),
    reason TEXT NOT NULL,

    UNIQUE (page_id, deleted_at)
);

--
-- Page backlinks tracking
--

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

    -- Rust enum: ConnectionType
    connection_type TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    count INT NOT NULL CHECK (count > 0),

    -- Enum value must not be empty
    CHECK (length(connection_type) > 0),

    PRIMARY KEY (from_page_id, to_page_id, connection_type)
);

CREATE TABLE page_connection_missing (
    from_page_id BIGINT REFERENCES page(page_id),
    to_site_id BIGINT REFERENCES site(site_id),
    to_page_slug TEXT,

    -- Rust enum: ConnectionType
    connection_type TEXT, -- Ditto
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    count INT NOT NULL CHECK (count > 0),

    -- Enum value must not be empty
    CHECK (length(connection_type) > 0),

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
    disabled_by BIGINT REFERENCES known_user(user_id),
    from_wikidot BOOLEAN NOT NULL DEFAULT false,
    page_id BIGINT NOT NULL REFERENCES page(page_id),
    user_id BIGINT NOT NULL REFERENCES known_user(user_id),
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
    created_by BIGINT NOT NULL REFERENCES known_user(user_id),
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

-- Manages blobs which are prohibited from being uploaded
CREATE TABLE blob_blacklist (
    s3_hash BYTEA PRIMARY KEY CHECK (length(s3_hash) = 64),  -- SHA-512 hash size
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    created_by BIGINT NOT NULL REFERENCES known_user(user_id)
);

--
-- Files
--

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

    -- Rust enum: FileRevisionType
    revision_type TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    revision_number INTEGER NOT NULL,
    file_id BIGINT NOT NULL REFERENCES file(file_id),
    page_id BIGINT NOT NULL REFERENCES page(page_id),
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    user_id BIGINT NOT NULL REFERENCES known_user(user_id),
    name TEXT NOT NULL,
    s3_hash BYTEA NOT NULL,
    mime TEXT NOT NULL,
    size BIGINT NOT NULL,
    changes TEXT[] NOT NULL DEFAULT '{}', -- List of changes in this revision
    comments TEXT NOT NULL,
    hidden TEXT[] NOT NULL DEFAULT '{}', -- List of fields to be hidden/suppressed

    CHECK (length(name) > 0 AND length(name) < 256),  -- Constrain filename length
    CHECK (length(s3_hash) = 64),                     -- SHA-512 hash size
    CHECK (mime != ''),                               -- Should have a MIME hint

    -- Ensure array only contains valid values
    -- Change this to use the 'page_revision_change' type later
    CHECK (changes <@ '{
        page,
        name,
        blob,
        mime
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
            mime
        }'
    ),

    -- Enum value must not be empty
    CHECK (length(revision_type) > 0),

    -- Ensure array is not empty for regular revisions
    CHECK (revision_type NOT IN ('regular', 'rollback') OR changes != '{}'),

    -- Ensure page creations are always the first revision
    CHECK (revision_number != 0 OR revision_type = 'create'),

    -- For logical consistency, and adding an index
    UNIQUE (file_id, page_id, revision_number)
);

--
-- Hosted Text Blocks
--

CREATE TABLE text_block (
    -- Rust enum: TextBlockType
    block_type TEXT NOT NULL,
    page_id BIGINT NOT NULL REFERENCES page(page_id),
    block_index SMALLINT NOT NULL CHECK (block_index > 0),
    block_name TEXT CHECK (length(block_name) > 0),
    text_type TEXT,

    -- Enum value must not be empty
    CHECK (length(block_type) > 0),

    PRIMARY KEY (block_type, page_id, block_index),
    UNIQUE (page_id, block_name)
);

--
-- Authorization Tokens
--

CREATE TABLE authorization_token (
    token_id SERIAL PRIMARY KEY,
    token_value TEXT NOT NULL UNIQUE,
    created_by BIGINT NOT NULL REFERENCES known_user(user_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    description TEXT NOT NULL,

    CONSTRAINT token_value_valid CHECK (token_value SIMILAR TO '[A-Z]-[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}')
);

--
-- Direct Messages
--

-- A "record" is the underlying message data, with its contents, attachments,
-- and associated metadata such as sender and recipient(s).
CREATE TABLE message_record (
    external_id TEXT PRIMARY KEY, -- ID comes from message_draft
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    drafted_at TIMESTAMP WITH TIME ZONE NOT NULL,
    retracted_at TIMESTAMP WITH TIME ZONE,
    sender_id BIGINT NOT NULL REFERENCES known_user(user_id),

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
    user_id BIGINT NOT NULL REFERENCES known_user(user_id),  -- The user who owns the copy of this record

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
    recipient_id BIGINT NOT NULL REFERENCES known_user(user_id),

    -- Rust enum: MessageRecipientType
    recipient_type TEXT NOT NULL,

    PRIMARY KEY (record_id, recipient_id, recipient_type),

    -- Enum value must not be empty
    CHECK (length(recipient_type) > 0)
);

CREATE TABLE message_draft (
    external_id TEXT PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    user_id BIGINT NOT NULL REFERENCES known_user(user_id),
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

--
-- Forums
--

-- Groups contain categories, and are site-local.
CREATE TABLE forum_group (
    forum_group_id BIGSERIAL PRIMARY KEY,
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    created_by BIGINT NOT NULL REFERENCES known_user(user_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_by BIGINT REFERENCES known_user(user_id),
    updated_at TIMESTAMP WITH TIME ZONE,
    deleted_by BIGINT REFERENCES known_user(user_id),
    deleted_at TIMESTAMP WITH TIME ZONE,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    visible BOOLEAN NOT NULL DEFAULT true,
    sort_index INTEGER NOT NULL CHECK (sort_index >= 0),
    from_wikidot BOOLEAN NOT NULL DEFAULT false,

    UNIQUE (forum_group_id, site_id),
    CHECK ((updated_by IS NULL) = (updated_at IS NULL)),
    CHECK ((deleted_by IS NULL) = (deleted_at IS NULL))
);

-- Categories belong to a group, and are site-local.
CREATE TABLE forum_category (
    forum_category_id BIGSERIAL PRIMARY KEY,
    forum_group_id BIGINT NOT NULL REFERENCES forum_group(forum_group_id),
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    created_by BIGINT NOT NULL REFERENCES known_user(user_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_by BIGINT REFERENCES known_user(user_id),
    updated_at TIMESTAMP WITH TIME ZONE,
    deleted_by BIGINT REFERENCES known_user(user_id),
    deleted_at TIMESTAMP WITH TIME ZONE,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    sort_index INTEGER NOT NULL CHECK (sort_index >= 0),
    from_wikidot BOOLEAN NOT NULL DEFAULT false,

    -- Category settings:
    max_nest_level SMALLINT CHECK (0 <= max_nest_level AND max_nest_level <= 10),
    per_page_discussion BOOLEAN DEFAULT false,
    layout TEXT,

    -- Required for (forum_category_id, site_id) composite FKs from denormalized child rows.
    UNIQUE (forum_category_id, site_id),
    CHECK ((updated_by IS NULL) = (updated_at IS NULL)),
    CHECK ((deleted_by IS NULL) = (deleted_at IS NULL)),
    FOREIGN KEY (forum_group_id, site_id) REFERENCES forum_group(forum_group_id, site_id)
);

-- Threads live in a category (but can be moved between them).
CREATE TABLE forum_thread (
    forum_thread_id BIGSERIAL PRIMARY KEY,
    forum_category_id BIGINT NOT NULL REFERENCES forum_category(forum_category_id),
    forum_group_id BIGINT NOT NULL REFERENCES forum_group(forum_group_id),
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    page_id BIGINT REFERENCES page(page_id) UNIQUE,  -- For page discussion threads (NULL = regular thread)
    created_by BIGINT NOT NULL REFERENCES known_user(user_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_by BIGINT REFERENCES known_user(user_id),
    updated_at TIMESTAMP WITH TIME ZONE,
    deleted_by BIGINT REFERENCES known_user(user_id),
    deleted_at TIMESTAMP WITH TIME ZONE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    from_wikidot BOOLEAN NOT NULL DEFAULT false,
    sticky BOOLEAN NOT NULL DEFAULT false,

    CHECK ((updated_by IS NULL) = (updated_at IS NULL)),
    CHECK ((deleted_by IS NULL) = (deleted_at IS NULL)),
    -- Required for (forum_thread_id, site_id) composite FKs from denormalized child rows.
    UNIQUE (forum_thread_id, site_id),
    FOREIGN KEY (forum_category_id, site_id) REFERENCES forum_category(forum_category_id, site_id),
    FOREIGN KEY (forum_group_id, site_id) REFERENCES forum_group(forum_group_id, site_id)
);

-- Locks on threads (one active lock per thread at a time).
CREATE TABLE forum_thread_lock (
    forum_thread_lock_id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    from_wikidot BOOLEAN NOT NULL DEFAULT false,
    forum_thread_id BIGINT NOT NULL REFERENCES forum_thread(forum_thread_id),
    user_id BIGINT NOT NULL REFERENCES known_user(user_id),
    reason TEXT NOT NULL,
    lock_type TEXT NOT NULL,
    allow_new_posts BOOLEAN NOT NULL DEFAULT false,
    allow_post_edits BOOLEAN NOT NULL DEFAULT false,
    allow_post_deletions BOOLEAN NOT NULL DEFAULT false,

    UNIQUE (forum_thread_id, deleted_at)
);

-- Posts within a thread, optionally nested.
CREATE TABLE forum_post (
    forum_post_id BIGSERIAL PRIMARY KEY,
    parent_post_id BIGINT REFERENCES forum_post(forum_post_id),
    forum_thread_id BIGINT NOT NULL REFERENCES forum_thread(forum_thread_id),
    forum_category_id BIGINT NOT NULL REFERENCES forum_category(forum_category_id),
    forum_group_id BIGINT NOT NULL REFERENCES forum_group(forum_group_id),
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    user_id BIGINT NOT NULL REFERENCES known_user(user_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    deleted_by BIGINT REFERENCES known_user(user_id),
    deleted_at TIMESTAMP WITH TIME ZONE,
    from_wikidot BOOLEAN NOT NULL DEFAULT false,
    latest_revision_id BIGINT,

    CHECK ((deleted_by IS NULL) = (deleted_at IS NULL)),
    -- Required for (forum_post_id, site_id) composite FKs from denormalized child rows.
    UNIQUE (forum_post_id, site_id),
    FOREIGN KEY (forum_thread_id, site_id) REFERENCES forum_thread(forum_thread_id, site_id),
    FOREIGN KEY (forum_category_id, site_id) REFERENCES forum_category(forum_category_id, site_id),
    FOREIGN KEY (forum_group_id, site_id) REFERENCES forum_group(forum_group_id, site_id)
);

-- Locks on posts (one active lock per post at a time).
CREATE TABLE forum_post_lock (
    forum_post_lock_id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    forum_post_id BIGINT NOT NULL REFERENCES forum_post(forum_post_id),
    user_id BIGINT NOT NULL REFERENCES known_user(user_id),
    reason TEXT NOT NULL,
    lock_type TEXT NOT NULL,
    cascading BOOLEAN NOT NULL,

    UNIQUE (forum_post_id, deleted_at)
);

-- Revisions of posts.
CREATE TABLE forum_post_revision (
    forum_post_revision_id BIGSERIAL PRIMARY KEY,
    forum_post_id BIGINT NOT NULL REFERENCES forum_post(forum_post_id),
    forum_thread_id BIGINT NOT NULL REFERENCES forum_thread(forum_thread_id),
    forum_category_id BIGINT NOT NULL REFERENCES forum_category(forum_category_id),
    forum_group_id BIGINT NOT NULL REFERENCES forum_group(forum_group_id),
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    user_id BIGINT NOT NULL REFERENCES known_user(user_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    revision_number INTEGER NOT NULL CHECK (revision_number >= 0),
    from_wikidot BOOLEAN NOT NULL DEFAULT false,
    title TEXT NOT NULL,
    wikitext_hash BYTEA NOT NULL REFERENCES text(hash),
    compiled_html_hash BYTEA NOT NULL REFERENCES text(hash),
    compiled_at TIMESTAMP WITH TIME ZONE NOT NULL,
    compiled_generator TEXT NOT NULL,
    comments TEXT NOT NULL,

    UNIQUE (forum_post_id, revision_number),
    FOREIGN KEY (forum_thread_id, site_id) REFERENCES forum_thread(forum_thread_id, site_id),
    FOREIGN KEY (forum_category_id, site_id) REFERENCES forum_category(forum_category_id, site_id),
    FOREIGN KEY (forum_group_id, site_id) REFERENCES forum_group(forum_group_id, site_id),
    FOREIGN KEY (forum_post_id, site_id) REFERENCES forum_post(forum_post_id, site_id)
);

-- Latest revision FK on posts, now that the revision table exists.
ALTER TABLE forum_post
    ADD CONSTRAINT forum_post_latest_revision_fk
        FOREIGN KEY (latest_revision_id) REFERENCES forum_post_revision(forum_post_revision_id);

-- Pages can point to their dedicated discussion threads.
ALTER TABLE page
    ADD CONSTRAINT page_discussion_thread_fk
        FOREIGN KEY (discussion_thread_id) REFERENCES forum_thread(forum_thread_id);

-- Forum indexes
-- Keep sort order unique only among active rows; deleted rows should not reserve positions.
CREATE UNIQUE INDEX forum_group_sort_active_unique_idx ON forum_group (site_id, sort_index)
    WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX forum_category_sort_active_unique_idx ON forum_category (forum_group_id, sort_index)
    WHERE deleted_at IS NULL;
CREATE INDEX forum_group_sort_idx ON forum_group (site_id, sort_index);
CREATE INDEX forum_category_sort_idx ON forum_category (forum_group_id, sort_index);
CREATE INDEX forum_category_site_sort_idx ON forum_category (site_id, sort_index);
CREATE INDEX forum_thread_activity_idx ON forum_thread (forum_category_id, sticky DESC, COALESCE(updated_at, created_at) DESC);
CREATE INDEX forum_thread_created_idx ON forum_thread (forum_category_id, created_at DESC);
CREATE INDEX forum_post_thread_created_idx ON forum_post (forum_thread_id, created_at);
CREATE INDEX forum_post_parent_idx ON forum_post (parent_post_id);
CREATE INDEX forum_post_latest_revision_idx ON forum_post (latest_revision_id);
CREATE INDEX forum_post_revision_lookup_idx ON forum_post_revision (forum_post_id, revision_number DESC);

--
-- Role / permission system
--

-- Roles in a site.
CREATE TABLE role (
    role_id BIGSERIAL PRIMARY KEY,

    -- Denotes a unique role in a site. A user may be an admin in one site but a regular site member in another.
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    from_wikidot BOOLEAN NOT NULL DEFAULT false,

    -- Virtual roles are invisible to the user, granted by the system under specific conditions
    -- e.g. Guest role granted when a non-member visits the site,
    -- or Author role granted when a user interacts with a page they authored.
    -- Virtual roles are visible to admins where they can select the role's permissions.
    -- Virtual roles cannot be manually assigned.
    is_virtual BOOLEAN NOT NULL DEFAULT false,

    -- The parent role for this role, following a tree-based hierarchy.
    parent_role_id BIGINT REFERENCES role(role_id),

    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE,

    UNIQUE (site_id, name)
);

-- Role permissions (1-to-many)
CREATE TABLE role_permission (
    permission_id BIGSERIAL PRIMARY KEY,
    role_id BIGINT NOT NULL REFERENCES role(role_id),
    -- Denormalized to avoid a join when filtering permissions by site.
    site_id BIGINT NOT NULL REFERENCES site(site_id),
    resource_type TEXT NOT NULL,
    -- Polymorphic reference to a resource category. For example, if the resource_type is "forum_category", then this references forum_category_id.
    -- NULL means the permission applies to all resources of the given type
    resource_category_id BIGINT,
    action TEXT NOT NULL,
    UNIQUE (site_id, role_id, resource_type, resource_category_id, action)
);

CREATE INDEX idx_site_perms ON role_permission (site_id, resource_type, resource_category_id, action);

-- User role assignments (many-to-many)
CREATE TABLE user_role (
    user_id BIGINT NOT NULL REFERENCES known_user(user_id),
    role_id BIGINT NOT NULL REFERENCES role(role_id),
    -- Denormalized FK to avoid a join. (Remember, roles are tied to a specific site)
    site_id BIGINT NOT NULL REFERENCES site(site_id),

    assigned_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    assigned_by BIGINT NOT NULL REFERENCES known_user(user_id),
    expires_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE,
    PRIMARY KEY (user_id, role_id)
);

--
-- Audit Log
--

-- This very large table contains the platform's audit log.
--
-- Like relations, it is an algebraic data type, all rows
-- with the same event_type have the same data fields. These
-- columns are nullable to accomodate this.
--
-- Observe that there are no foreign keys here:
-- While we maintain that constraint in code, we are not
-- burdening Postgres with maintaining extremely large numbers
-- of these foreign keys.
--
CREATE TABLE audit_log (
    event_id BIGSERIAL PRIMARY KEY,
    event_type TEXT NOT NULL,  -- check enum value at runtime
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    ip_address TEXT NOT NULL,  -- TODO change to INET
    user_id BIGINT,
    site_id BIGINT,
    page_id BIGINT,
    extra_id_1 BIGINT,
    extra_id_2 BIGINT,
    extra_string_1 TEXT,
    extra_string_2 TEXT,
    extra_number INT,

    CHECK (strpos(event_type, '.') != 0)  -- all event types are '[object].[operation]'
);
