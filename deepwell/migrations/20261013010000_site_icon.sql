-- Site-level icon assets (favicon, iOS touch icons, Windows tile), stored per
-- site and kind outside the page model. File rows require a page, so icons
-- cannot live there; these bytes back the /local--favicon/*, /local--iosicon/*,
-- and /local--wp8icon/* compatibility routes without depending on Wikidot.

CREATE TABLE site_icon (
    site_id BIGINT NOT NULL REFERENCES site (site_id) ON DELETE CASCADE,
    kind TEXT NOT NULL
        CHECK (kind IN ('favicon', 'ios', 'windows')),
    mime TEXT NOT NULL
        CHECK (mime IN ('image/gif', 'image/png', 'image/x-icon', 'image/jpeg')),
    bytes BYTEA NOT NULL
        CHECK (octet_length(bytes) BETWEEN 1 AND 524288),
    sha256_hex TEXT NOT NULL
        CHECK (sha256_hex ~ '^[0-9a-f]{64}$'),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (site_id, kind)
);
