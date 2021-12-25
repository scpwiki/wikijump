SET default_transaction_read_only = off;

CREATE ROLE wikijump
    WITH INHERIT NOSUPERUSER CREATEDB LOGIN REPLICATION NOBYPASSRLS PASSWORD 'wikijump';

CREATE ROLE wikijump_ro
    WITH INHERIT NOSUPERUSER NOCREATEDB LOGIN NOREPLICATION NOBYPASSRLS PASSWORD 'wikijump_ro';

CREATE DATABASE wikijump ENCODING = 'UTF8' LC_COLLATE = 'en_US.UTF-8' LC_CTYPE = 'en_US.UTF-8';
ALTER DATABASE wikijump OWNER TO wikijump;

\connect wikijump

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

-- Since permissions are additive, and by default
-- everyone in public can create tables, we have
-- to revoke it from everyone so we can create a
-- read-only user.
--
-- The first 'public' is the schema,
-- the second 'PUBLIC' means "all users".
REVOKE CREATE ON SCHEMA public FROM PUBLIC, wikijump_ro;

-- Then we give all permissions to wikijump so it can do things.
GRANT ALL ON SCHEMA public TO wikijump;

-- Revoke all permissions, except SELECT.
-- Also grant SELECT on any future tables that are made.
REVOKE ALL ON SCHEMA public FROM wikijump_ro;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO wikijump_ro;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT SELECT ON TABLES TO wikijump_ro;
