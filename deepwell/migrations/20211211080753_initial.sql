-- Initial migration script
--
-- This doesn't actually do anything. The service is starting in a
-- weird place where it shares custody of the database with Laravel.
-- For consistency, all migrations will be via Eloquent (using raw
-- queries) as table ownership is slowly passed to DEEPWELL.
--
-- Then, after ownership of all has been passed, we can move them all
-- here and remove the old Laravel migration infrastructure and run
-- solely from this directory.

SELECT 1;
