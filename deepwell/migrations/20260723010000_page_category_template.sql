-- Wikidot page templates are named pages in the template: category assigned as the initial editor source for newly created pages in another category.
ALTER TABLE page_category
    ADD COLUMN template_page_id BIGINT REFERENCES page(page_id) ON DELETE SET NULL;
