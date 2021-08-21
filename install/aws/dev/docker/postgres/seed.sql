SET default_transaction_read_only = off;

CREATE ROLE wikijump;
ALTER ROLE wikijump WITH INHERIT CREATEDB LOGIN REPLICATION NOBYPASSRLS PASSWORD 'wikijump';

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

GRANT ALL ON SCHEMA public TO wikijump;

/* Temp password, this will become a Terraform secret, don't @ me */
create user datadog with password 'Ge07mcovAKvIT9WM';
grant pg_monitor to datadog;
grant SELECT ON pg_stat_database to datadog;
