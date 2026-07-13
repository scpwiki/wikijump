ALTER TABLE text
    ADD COLUMN IF NOT EXISTS character_count BIGINT
    GENERATED ALWAYS AS (char_length(contents)::BIGINT) STORED;

CREATE INDEX IF NOT EXISTS text_character_count_idx
    ON text (character_count);
