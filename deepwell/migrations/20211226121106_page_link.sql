-- This takes ownership of page_link, page_connection, and page_connection_missing tables.

DROP TABLE page_link;
CREATE TABLE page_link (
    page_id BIGINT,
    url TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    edited_at TIMESTAMP WITH TIME ZONE,
    count INT NOT NULL CHECK (count > 0),

    PRIMARY KEY (page_id, url)
);

DROP TABLE page_connection;
CREATE TABLE page_connection (
    from_page_id BIGINT,
    to_page_id BIGINT,
    connection_type TEXT
        CHECK (connection_type = ANY(ARRAY[
            'include-messy',
            'include-elements',
            'component',
            'link',
            'redirect'
        ])),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    edited_at TIMESTAMP WITH TIME ZONE,
    count INT NOT NULL CHECK (count > 0),

    PRIMARY KEY (from_page_id, to_page_id, connection_type)
);

DROP TABLE page_connection_missing;
CREATE TABLE page_connection_missing (
    from_page_id BIGINT,
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
    edited_at TIMESTAMP WITH TIME ZONE,
    count INT NOT NULL CHECK (count > 0),

    PRIMARY KEY (from_page_id, to_page_slug, connection_type)
);
