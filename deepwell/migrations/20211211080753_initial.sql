-- Initial migration script
--
-- This doesn't actually do anything. The service is starting in a
-- weird place where it shares custody of the database with Laravel,
-- so it has tables pre-existing in the database from its migration there.
-- You need to ensure all of those are run (up to 2021_11_28_*) before running
-- the migrations here.

SELECT 1;
