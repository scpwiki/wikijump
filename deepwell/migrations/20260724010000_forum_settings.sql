ALTER TABLE site
    ADD COLUMN forum_max_nest_level SMALLINT NOT NULL DEFAULT 10
        CHECK (forum_max_nest_level BETWEEN 0 AND 10);

ALTER TABLE page_category
    ADD COLUMN per_page_discussion BOOLEAN;
