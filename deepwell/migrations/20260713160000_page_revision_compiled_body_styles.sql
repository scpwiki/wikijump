-- Preserve renderer-provenanced page CSS independently from compiled body HTML.
-- Existing revisions remain readable and expose no generated styles until rerendered.
ALTER TABLE page_revision
    ADD COLUMN compiled_body_styles_hash BYTEA REFERENCES text(hash);
