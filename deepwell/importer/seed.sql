CREATE TABLE blob (
    hex_hash TEXT PRIMARY KEY,
    length INTEGER NOT NULL
);

CREATE TABLE text (
    hex_hash TEXT PRIMARY KEY,
    contents TEXT NOT NULL
);

CREATE TABLE site (
    site_slug TEXT PRIMARY KEY,
    site_descr TEXT NOT NULL,  -- Wikicomma name
    site_url TEXT NOT NULL,
    site_id INTEGER NOT NULL
);

CREATE TABLE page (
    page_id INTEGER PRIMARY KEY,
    site_slug TEXT NOT NULL REFERENCES site(site_slug),
    page_slug TEXT NOT NULL,

    UNIQUE (site_slug, page_slug)
);

CREATE TABLE page_metadata (
    page_id INTEGER PRIMARY KEY REFERENCES page(page_id),
    page_descr TEXT NOT NULL,
    sitemap_updated_at INTEGER NOT NULL,
    title TEXT NOT NULL,
    locked INTEGER NOT NULL CHECK (locked IN (0, 1)),  -- boolean
    tags TEXT NOT NULL  -- JSON
);

CREATE TABLE page_revision (
    revision_id INTEGER PRIMARY KEY,
    revision_number INTEGER NOT NULL CHECK (revision_number >= 0),
    page_id INTEGER NOT NULL REFERENCES page(page_id),
    user_id INTEGER NOT NULL REFERENCES user(user_id),
    created_at INTEGER NOT NULL,
    flags TEXT NOT NULL,
    comments TEXT NOT NULL,

    UNIQUE (page_id, revision_number)
);

CREATE TABLE page_revision_wikitext (
    revision_id INTEGER PRIMARY KEY REFERENCES page_revision(revision_id),
    wikitext_hash TEXT NOT NULL REFERENCES text(hex_hash)
);

CREATE TABLE page_vote (
    page_id INTEGER REFERENCES page(page_id),
    user_id INTEGER REFERENCES user(user_id),
    value INTEGER NOT NULL,

    PRIMARY KEY (page_id, user_id)
);

CREATE TABLE file (
    file_id INTEGER PRIMARY KEY,
    page_id INTEGER NOT NULL REFERENCES page(page_id),
    site_slug TEXT NOT NULL REFERENCES site(site_slug)
);

CREATE TABLE user (
    user_slug TEXT PRIMARY KEY,
    user_name TEXT NOT NULL,
    user_id INTEGER NOT NULL UNIQUE,
    user_since INTEGER NOT NULL,
    account_type TEXT NOT NULL,
    karma INTEGER NOT NULL,
    fetched_at INTEGER NOT NULL,
    real_name TEXT,
    gender TEXT,
    birthday INTEGER,
    location TEXT,
    website TEXT
);
