CREATE TABLE blob (
    hex_hash TEXT PRIMARY KEY,
    length INTEGER NOT NULL
);

CREATE TABLE site (
    site_slug TEXT PRIMARY KEY,
    site_descr TEXT NOT NULL,  -- Wikicomma name
    site_url TEXT NOT NULL,
    site_id INTEGER NOT NULL
);

CREATE TABLE page (
    site_slug TEXT NOT NULL REFERENCES site(site_slug),
    page_slug TEXT NOT NULL,

    PRIMARY KEY (site_slug, page_slug)
);

CREATE TABLE file (

    site_slug TEXT NOT NULL REFERENCES site(site_slug),
    page_slug TEXT NOT NULL,

    FOREIGN KEY (site_slug, page_slug) REFERENCES page(site_slug, page_slug)
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
