ALTER TABLE page_category
    ADD COLUMN rating_enabled BOOLEAN,
    ADD COLUMN rating_permission TEXT
        CHECK (rating_permission IN ('registered', 'members')),
    ADD COLUMN rating_visibility TEXT
        CHECK (rating_visibility IN ('visible', 'anonymous')),
    ADD COLUMN rating_type TEXT
        CHECK (rating_type IN ('plus', 'plus_minus', 'stars'));

ALTER TABLE page_vote
    ADD COLUMN rating_system TEXT NOT NULL DEFAULT 'points'
        CHECK (rating_system IN ('points', 'stars'));
