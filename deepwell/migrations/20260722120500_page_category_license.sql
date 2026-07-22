-- Wikidot licenses are category-scoped. NULL inherits the `_default` category,
-- with the site license retained as the legacy fallback.
ALTER TABLE page_category
    ADD COLUMN license TEXT,
    ADD COLUMN license_other TEXT;

-- The Wikidot _default category is the inheritance source. Preserve each
-- existing site's current license as that explicit category baseline.
UPDATE page_category AS category
SET license = site.license
FROM site
WHERE category.site_id = site.site_id
  AND category.slug = '_default';
