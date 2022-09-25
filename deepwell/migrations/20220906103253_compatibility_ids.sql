-- Modify ID sequences so that they exhibit Wikidot compatibility.
--
-- This property means that no valid Wikidot ID for a class of object
-- can ever also be a valid Wikijump ID for that same class of object.
-- We do this by putting the start ID for new Wikijump IDs well above
-- what the Wikidot value is likely to reach by the time the project
-- hits production.
--
-- Some classes of object are not assigned compatibility IDs, either
-- because the ID value does not matter, is unused, or is not exposed.
--
-- See https://scuttle.atlassian.net/browse/WJ-964

ALTER SEQUENCE user_user_id_seq                 RESTART WITH   10000000;
ALTER SEQUENCE site_site_id_seq                 RESTART WITH    6000000;
ALTER SEQUENCE page_page_id_seq                 RESTART WITH 3000000000;
ALTER SEQUENCE page_revision_revision_id_seq    RESTART WITH 3000000000;
-- TODO: ALTER SEQUENCE -forum category-                 RESTART WITH    9000000;
-- TODO: ALTER SEQUENCE -forum thread-                   RESTART WITH   30000000;
-- TODO: ALTER SEQUENCE -forum post-                     RESTART WITH    7000000;
