--
-- PostgreSQL database cluster dump {{{
--

SET default_transaction_read_only = off;

SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;

--
-- Roles
--

CREATE ROLE wikijump;
ALTER ROLE wikijump WITH INHERIT CREATEDB LOGIN REPLICATION NOBYPASSRLS PASSWORD 'wikijump';

--
-- Databases
--

--
-- Database "template1" dump {{{
--

\connect template1

--
-- PostgreSQL database dump
--

-- Dumped from database version 12.2 (Ubuntu 12.2-4)
-- Dumped by pg_dump version 12.2 (Ubuntu 12.2-4)

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

--
-- PostgreSQL database dump complete }}}
--

--
-- Database "postgres" dump {{{
--

\connect postgres

--
-- PostgreSQL database dump
--

-- Dumped from database version 12.2 (Ubuntu 12.2-4)
-- Dumped by pg_dump version 12.2 (Ubuntu 12.2-4)

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

--
-- Name: ts2; Type: SCHEMA; Schema: -; Owner: postgres
--

CREATE SCHEMA ts2;


ALTER SCHEMA ts2 OWNER TO postgres;

--
-- Name: gtsq; Type: DOMAIN; Schema: ts2; Owner: postgres
--

CREATE DOMAIN ts2.gtsq AS text;


ALTER DOMAIN ts2.gtsq OWNER TO postgres;

--
-- Name: gtsvector; Type: DOMAIN; Schema: ts2; Owner: postgres
--

CREATE DOMAIN ts2.gtsvector AS gtsvector;


ALTER DOMAIN ts2.gtsvector OWNER TO postgres;

--
-- Name: statinfo; Type: TYPE; Schema: ts2; Owner: postgres
--

CREATE TYPE ts2.statinfo AS (
    word text,
    ndoc integer,
    nentry integer
);


ALTER TYPE ts2.statinfo OWNER TO postgres;

--
-- Name: tokenout; Type: TYPE; Schema: ts2; Owner: postgres
--

CREATE TYPE ts2.tokenout AS (
    tokid integer,
    token text
);


ALTER TYPE ts2.tokenout OWNER TO postgres;

--
-- Name: tokentype; Type: TYPE; Schema: ts2; Owner: postgres
--

CREATE TYPE ts2.tokentype AS (
    tokid integer,
    alias text,
    descr text
);


ALTER TYPE ts2.tokentype OWNER TO postgres;

--
-- Name: tsdebug; Type: TYPE; Schema: ts2; Owner: postgres
--

CREATE TYPE ts2.tsdebug AS (
    ts_name text,
    tok_type text,
    description text,
    token text,
    dict_name text[],
    tsvector tsvector
);


ALTER TYPE ts2.tsdebug OWNER TO postgres;

--
-- Name: tsquery; Type: DOMAIN; Schema: ts2; Owner: postgres
--

CREATE DOMAIN ts2.tsquery AS tsquery;


ALTER DOMAIN ts2.tsquery OWNER TO postgres;

--
-- Name: tsvector; Type: DOMAIN; Schema: ts2; Owner: postgres
--

CREATE DOMAIN ts2.tsvector AS tsvector;


ALTER DOMAIN ts2.tsvector OWNER TO postgres;

--
-- Name: _get_parser_from_curcfg(); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2._get_parser_from_curcfg() RETURNS text
    LANGUAGE sql IMMUTABLE STRICT
    AS $$select prsname::text from pg_catalog.pg_ts_parser p join pg_ts_config c on cfgparser = p.oid where c.oid = show_curcfg();$$;


ALTER FUNCTION ts2._get_parser_from_curcfg() OWNER TO postgres;

--
-- Name: concat(tsvector, tsvector); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.concat(tsvector, tsvector) RETURNS tsvector
    LANGUAGE internal IMMUTABLE STRICT
    AS $$tsvector_concat$$;


ALTER FUNCTION ts2.concat(tsvector, tsvector) OWNER TO postgres;

--
-- Name: headline(text, tsquery); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.headline(text, tsquery) RETURNS text
    LANGUAGE internal IMMUTABLE STRICT
    AS $$ts_headline$$;


ALTER FUNCTION ts2.headline(text, tsquery) OWNER TO postgres;

--
-- Name: headline(oid, text, tsquery); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.headline(oid, text, tsquery) RETURNS text
    LANGUAGE internal IMMUTABLE STRICT
    AS $$ts_headline_byid$$;


ALTER FUNCTION ts2.headline(oid, text, tsquery) OWNER TO postgres;

--
-- Name: headline(text, tsquery, text); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.headline(text, tsquery, text) RETURNS text
    LANGUAGE internal IMMUTABLE STRICT
    AS $$ts_headline_opt$$;


ALTER FUNCTION ts2.headline(text, tsquery, text) OWNER TO postgres;

--
-- Name: headline(oid, text, tsquery, text); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.headline(oid, text, tsquery, text) RETURNS text
    LANGUAGE internal IMMUTABLE STRICT
    AS $$ts_headline_byid_opt$$;


ALTER FUNCTION ts2.headline(oid, text, tsquery, text) OWNER TO postgres;

--
-- Name: length(tsvector); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.length(tsvector) RETURNS integer
    LANGUAGE internal IMMUTABLE STRICT
    AS $$tsvector_length$$;


ALTER FUNCTION ts2.length(tsvector) OWNER TO postgres;

--
-- Name: lexize(oid, text); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.lexize(oid, text) RETURNS text[]
    LANGUAGE internal STRICT
    AS $$ts_lexize$$;


ALTER FUNCTION ts2.lexize(oid, text) OWNER TO postgres;

--
-- Name: numnode(tsquery); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.numnode(tsquery) RETURNS integer
    LANGUAGE internal IMMUTABLE STRICT
    AS $$tsquery_numnode$$;


ALTER FUNCTION ts2.numnode(tsquery) OWNER TO postgres;

--
-- Name: parse(oid, text); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.parse(oid, text) RETURNS SETOF ts2.tokenout
    LANGUAGE internal STRICT
    AS $$ts_parse_byid$$;


ALTER FUNCTION ts2.parse(oid, text) OWNER TO postgres;

--
-- Name: parse(text, text); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.parse(text, text) RETURNS SETOF ts2.tokenout
    LANGUAGE internal STRICT
    AS $$ts_parse_byname$$;


ALTER FUNCTION ts2.parse(text, text) OWNER TO postgres;

--
-- Name: plainto_tsquery(text); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.plainto_tsquery(text) RETURNS tsquery
    LANGUAGE internal IMMUTABLE STRICT
    AS $$plainto_tsquery$$;


ALTER FUNCTION ts2.plainto_tsquery(text) OWNER TO postgres;

--
-- Name: plainto_tsquery(oid, text); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.plainto_tsquery(oid, text) RETURNS tsquery
    LANGUAGE internal IMMUTABLE STRICT
    AS $$plainto_tsquery_byid$$;


ALTER FUNCTION ts2.plainto_tsquery(oid, text) OWNER TO postgres;

--
-- Name: querytree(tsquery); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.querytree(tsquery) RETURNS text
    LANGUAGE internal STRICT
    AS $$tsquerytree$$;


ALTER FUNCTION ts2.querytree(tsquery) OWNER TO postgres;

--
-- Name: rank(tsvector, tsquery); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.rank(tsvector, tsquery) RETURNS real
    LANGUAGE internal IMMUTABLE STRICT
    AS $$ts_rank_tt$$;


ALTER FUNCTION ts2.rank(tsvector, tsquery) OWNER TO postgres;

--
-- Name: rank(real[], tsvector, tsquery); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.rank(real[], tsvector, tsquery) RETURNS real
    LANGUAGE internal IMMUTABLE STRICT
    AS $$ts_rank_wtt$$;


ALTER FUNCTION ts2.rank(real[], tsvector, tsquery) OWNER TO postgres;

--
-- Name: rank(tsvector, tsquery, integer); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.rank(tsvector, tsquery, integer) RETURNS real
    LANGUAGE internal IMMUTABLE STRICT
    AS $$ts_rank_ttf$$;


ALTER FUNCTION ts2.rank(tsvector, tsquery, integer) OWNER TO postgres;

--
-- Name: rank(real[], tsvector, tsquery, integer); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.rank(real[], tsvector, tsquery, integer) RETURNS real
    LANGUAGE internal IMMUTABLE STRICT
    AS $$ts_rank_wttf$$;


ALTER FUNCTION ts2.rank(real[], tsvector, tsquery, integer) OWNER TO postgres;

--
-- Name: rank_cd(tsvector, tsquery); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.rank_cd(tsvector, tsquery) RETURNS real
    LANGUAGE internal IMMUTABLE STRICT
    AS $$ts_rankcd_tt$$;


ALTER FUNCTION ts2.rank_cd(tsvector, tsquery) OWNER TO postgres;

--
-- Name: rank_cd(real[], tsvector, tsquery); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.rank_cd(real[], tsvector, tsquery) RETURNS real
    LANGUAGE internal IMMUTABLE STRICT
    AS $$ts_rankcd_wtt$$;


ALTER FUNCTION ts2.rank_cd(real[], tsvector, tsquery) OWNER TO postgres;

--
-- Name: rank_cd(tsvector, tsquery, integer); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.rank_cd(tsvector, tsquery, integer) RETURNS real
    LANGUAGE internal IMMUTABLE STRICT
    AS $$ts_rankcd_ttf$$;


ALTER FUNCTION ts2.rank_cd(tsvector, tsquery, integer) OWNER TO postgres;

--
-- Name: rank_cd(real[], tsvector, tsquery, integer); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.rank_cd(real[], tsvector, tsquery, integer) RETURNS real
    LANGUAGE internal IMMUTABLE STRICT
    AS $$ts_rankcd_wttf$$;


ALTER FUNCTION ts2.rank_cd(real[], tsvector, tsquery, integer) OWNER TO postgres;

--
-- Name: rewrite(tsquery, text); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.rewrite(tsquery, text) RETURNS tsquery
    LANGUAGE internal IMMUTABLE STRICT
    AS $$tsquery_rewrite_query$$;


ALTER FUNCTION ts2.rewrite(tsquery, text) OWNER TO postgres;

--
-- Name: rewrite(tsquery, tsquery, tsquery); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.rewrite(tsquery, tsquery, tsquery) RETURNS tsquery
    LANGUAGE internal IMMUTABLE STRICT
    AS $$tsquery_rewrite$$;


ALTER FUNCTION ts2.rewrite(tsquery, tsquery, tsquery) OWNER TO postgres;

--
-- Name: setweight(tsvector, "char"); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.setweight(tsvector, "char") RETURNS tsvector
    LANGUAGE internal IMMUTABLE STRICT
    AS $$tsvector_setweight$$;


ALTER FUNCTION ts2.setweight(tsvector, "char") OWNER TO postgres;

--
-- Name: show_curcfg(); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.show_curcfg() RETURNS oid
    LANGUAGE internal STABLE STRICT
    AS $$get_current_ts_config$$;


ALTER FUNCTION ts2.show_curcfg() OWNER TO postgres;

--
-- Name: stat(text); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.stat(text) RETURNS SETOF ts2.statinfo
    LANGUAGE internal STRICT
    AS $$ts_stat1$$;


ALTER FUNCTION ts2.stat(text) OWNER TO postgres;

--
-- Name: stat(text, text); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.stat(text, text) RETURNS SETOF ts2.statinfo
    LANGUAGE internal STRICT
    AS $$ts_stat2$$;


ALTER FUNCTION ts2.stat(text, text) OWNER TO postgres;

--
-- Name: strip(tsvector); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.strip(tsvector) RETURNS tsvector
    LANGUAGE internal IMMUTABLE STRICT
    AS $$tsvector_strip$$;


ALTER FUNCTION ts2.strip(tsvector) OWNER TO postgres;

--
-- Name: to_tsquery(text); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.to_tsquery(text) RETURNS tsquery
    LANGUAGE internal IMMUTABLE STRICT
    AS $$to_tsquery$$;


ALTER FUNCTION ts2.to_tsquery(text) OWNER TO postgres;

--
-- Name: to_tsquery(oid, text); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.to_tsquery(oid, text) RETURNS tsquery
    LANGUAGE internal IMMUTABLE STRICT
    AS $$to_tsquery_byid$$;


ALTER FUNCTION ts2.to_tsquery(oid, text) OWNER TO postgres;

--
-- Name: to_tsvector(text); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.to_tsvector(text) RETURNS tsvector
    LANGUAGE internal IMMUTABLE STRICT
    AS $$to_tsvector$$;


ALTER FUNCTION ts2.to_tsvector(text) OWNER TO postgres;

--
-- Name: to_tsvector(oid, text); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.to_tsvector(oid, text) RETURNS tsvector
    LANGUAGE internal IMMUTABLE STRICT
    AS $$to_tsvector_byid$$;


ALTER FUNCTION ts2.to_tsvector(oid, text) OWNER TO postgres;

--
-- Name: token_type(integer); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.token_type(integer) RETURNS SETOF ts2.tokentype
    LANGUAGE internal STRICT ROWS 16
    AS $$ts_token_type_byid$$;


ALTER FUNCTION ts2.token_type(integer) OWNER TO postgres;

--
-- Name: token_type(text); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.token_type(text) RETURNS SETOF ts2.tokentype
    LANGUAGE internal STRICT ROWS 16
    AS $$ts_token_type_byname$$;


ALTER FUNCTION ts2.token_type(text) OWNER TO postgres;

--
-- Name: ts_debug(text); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.ts_debug(text) RETURNS SETOF ts2.tsdebug
    LANGUAGE sql STRICT
    AS $_$
select
        (select c.cfgname::text from pg_catalog.pg_ts_config as c
         where c.oid = show_curcfg()),
        t.alias as tok_type,
        t.descr as description,
        p.token,
        ARRAY ( SELECT m.mapdict::pg_catalog.regdictionary::pg_catalog.text
                FROM pg_catalog.pg_ts_config_map AS m
                WHERE m.mapcfg = show_curcfg() AND m.maptokentype = p.tokid
                ORDER BY m.mapseqno )
        AS dict_name,
        strip(to_tsvector(p.token)) as tsvector
from
        parse( _get_parser_from_curcfg(), $1 ) as p,
        token_type() as t
where
        t.tokid = p.tokid
$_$;


ALTER FUNCTION ts2.ts_debug(text) OWNER TO postgres;

--
-- Name: tsq_mcontained(tsquery, tsquery); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.tsq_mcontained(tsquery, tsquery) RETURNS boolean
    LANGUAGE internal IMMUTABLE STRICT
    AS $$tsq_mcontained$$;


ALTER FUNCTION ts2.tsq_mcontained(tsquery, tsquery) OWNER TO postgres;

--
-- Name: tsq_mcontains(tsquery, tsquery); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.tsq_mcontains(tsquery, tsquery) RETURNS boolean
    LANGUAGE internal IMMUTABLE STRICT
    AS $$tsq_mcontains$$;


ALTER FUNCTION ts2.tsq_mcontains(tsquery, tsquery) OWNER TO postgres;

--
-- Name: tsquery_and(tsquery, tsquery); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.tsquery_and(tsquery, tsquery) RETURNS tsquery
    LANGUAGE internal IMMUTABLE STRICT
    AS $$tsquery_and$$;


ALTER FUNCTION ts2.tsquery_and(tsquery, tsquery) OWNER TO postgres;

--
-- Name: tsquery_not(tsquery); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.tsquery_not(tsquery) RETURNS tsquery
    LANGUAGE internal IMMUTABLE STRICT
    AS $$tsquery_not$$;


ALTER FUNCTION ts2.tsquery_not(tsquery) OWNER TO postgres;

--
-- Name: tsquery_or(tsquery, tsquery); Type: FUNCTION; Schema: ts2; Owner: postgres
--

CREATE FUNCTION ts2.tsquery_or(tsquery, tsquery) RETURNS tsquery
    LANGUAGE internal IMMUTABLE STRICT
    AS $$tsquery_or$$;


ALTER FUNCTION ts2.tsquery_or(tsquery, tsquery) OWNER TO postgres;

--
-- Name: tsquery_ops; Type: OPERATOR FAMILY; Schema: ts2; Owner: postgres
--

CREATE OPERATOR FAMILY ts2.tsquery_ops USING btree;


ALTER OPERATOR FAMILY ts2.tsquery_ops USING btree OWNER TO postgres;

--
-- Name: tsquery_ops; Type: OPERATOR CLASS; Schema: ts2; Owner: postgres
--

CREATE OPERATOR CLASS ts2.tsquery_ops
    FOR TYPE tsquery USING btree FAMILY ts2.tsquery_ops AS
    OPERATOR 1 <(tsquery,tsquery) ,
    OPERATOR 2 <=(tsquery,tsquery) ,
    OPERATOR 3 =(tsquery,tsquery) ,
    OPERATOR 4 >=(tsquery,tsquery) ,
    OPERATOR 5 >(tsquery,tsquery) ,
    FUNCTION 1 (tsquery, tsquery) tsquery_cmp(tsquery,tsquery);


ALTER OPERATOR CLASS ts2.tsquery_ops USING btree OWNER TO postgres;

--
-- Name: tsvector_ops; Type: OPERATOR FAMILY; Schema: ts2; Owner: postgres
--

CREATE OPERATOR FAMILY ts2.tsvector_ops USING btree;


ALTER OPERATOR FAMILY ts2.tsvector_ops USING btree OWNER TO postgres;

--
-- Name: tsvector_ops; Type: OPERATOR CLASS; Schema: ts2; Owner: postgres
--

CREATE OPERATOR CLASS ts2.tsvector_ops
    FOR TYPE tsvector USING btree FAMILY ts2.tsvector_ops AS
    OPERATOR 1 <(tsvector,tsvector) ,
    OPERATOR 2 <=(tsvector,tsvector) ,
    OPERATOR 3 =(tsvector,tsvector) ,
    OPERATOR 4 >=(tsvector,tsvector) ,
    OPERATOR 5 >(tsvector,tsvector) ,
    FUNCTION 1 (tsvector, tsvector) tsvector_cmp(tsvector,tsvector);


ALTER OPERATOR CLASS ts2.tsvector_ops USING btree OWNER TO postgres;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Database "wikijump" dump
--

--
-- PostgreSQL database dump
--

-- Dumped from database version 12.2 (Ubuntu 12.2-4)
-- Dumped by pg_dump version 12.2 (Ubuntu 12.2-4)

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

--
-- PostgreSQL database dump complete }}}
--

--
-- Database "wikijump" dump {{{
--

--
-- Name: wikijump; Type: DATABASE; Schema: -; Owner: postgres
--

CREATE DATABASE wikijump WITH TEMPLATE = template0 ENCODING = 'UTF8' LC_COLLATE = 'en_US.UTF-8' LC_CTYPE = 'en_US.UTF-8';


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

--
-- Name: admin; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.admin (
    admin_id integer NOT NULL,
    site_id integer,
    user_id integer,
    founder boolean DEFAULT false
);


ALTER TABLE public.admin OWNER TO wikijump;

--
-- Name: admin_admin_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.admin_admin_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.admin_admin_id_seq OWNER TO wikijump;

--
-- Name: admin_admin_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.admin_admin_id_seq OWNED BY public.admin.admin_id;


--
-- Name: admin_notification; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.admin_notification (
    notification_id integer NOT NULL,
    site_id integer,
    body text,
    type character varying(50),
    viewed boolean DEFAULT false,
    date timestamp without time zone,
    extra bytea,
    notify_online boolean DEFAULT false,
    notify_feed boolean DEFAULT false,
    notify_email boolean DEFAULT false
);


ALTER TABLE public.admin_notification OWNER TO wikijump;

--
-- Name: admin_notification_notification_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.admin_notification_notification_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.admin_notification_notification_id_seq OWNER TO wikijump;

--
-- Name: admin_notification_notification_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.admin_notification_notification_id_seq OWNED BY public.admin_notification.notification_id;


--
-- Name: anonymous_abuse_flag; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.anonymous_abuse_flag (
    flag_id integer NOT NULL,
    user_id integer,
    address inet,
    proxy boolean DEFAULT false,
    site_id integer,
    site_valid boolean DEFAULT true,
    global_valid boolean DEFAULT true
);


ALTER TABLE public.anonymous_abuse_flag OWNER TO wikijump;

--
-- Name: anonymous_abuse_flag_flag_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.anonymous_abuse_flag_flag_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.anonymous_abuse_flag_flag_id_seq OWNER TO wikijump;

--
-- Name: anonymous_abuse_flag_flag_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.anonymous_abuse_flag_flag_id_seq OWNED BY public.anonymous_abuse_flag.flag_id;


--
-- Name: category; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.category (
    category_id integer NOT NULL,
    site_id integer,
    name character varying(80),
    theme_default boolean DEFAULT true,
    theme_id integer,
    permissions_default boolean DEFAULT true,
    permissions character varying(200),
    license_default boolean DEFAULT true,
    license_id integer,
    license_other character varying(350),
    nav_default boolean DEFAULT true,
    top_bar_page_name character varying(128),
    side_bar_page_name character varying(128),
    template_id integer,
    per_page_discussion boolean,
    per_page_discussion_default boolean DEFAULT true,
    rating character varying(10),
    category_template_id integer,
    theme_external_url character varying(512),
    enable_pingback_out boolean DEFAULT true,
    enable_pingback_in boolean DEFAULT false,
    autonumerate boolean DEFAULT false,
    page_title_template character varying(256)
);


ALTER TABLE public.category OWNER TO wikijump;

--
-- Name: category_category_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.category_category_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.category_category_id_seq OWNER TO wikijump;

--
-- Name: category_category_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.category_category_id_seq OWNED BY public.category.category_id;


--
-- Name: category_template; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.category_template (
    category_template_id integer NOT NULL,
    source text
);


ALTER TABLE public.category_template OWNER TO wikijump;

--
-- Name: category_template_category_template_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.category_template_category_template_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.category_template_category_template_id_seq OWNER TO wikijump;

--
-- Name: category_template_category_template_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.category_template_category_template_id_seq OWNED BY public.category_template.category_template_id;


--
-- Name: comment; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.comment (
    comment_id integer NOT NULL,
    page_id integer,
    parent_id integer,
    user_id integer,
    user_string character varying(80),
    title character varying(256),
    text text,
    date_posted timestamp without time zone,
    site_id integer,
    revision_number integer DEFAULT 0,
    revision_id integer,
    date_last_edited timestamp without time zone,
    edited_user_id integer,
    edited_user_string character varying(80)
);


ALTER TABLE public.comment OWNER TO wikijump;

--
-- Name: comment_comment_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.comment_comment_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.comment_comment_id_seq OWNER TO wikijump;

--
-- Name: comment_comment_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.comment_comment_id_seq OWNED BY public.comment.comment_id;


--
-- Name: comment_revision; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.comment_revision (
    revision_id integer NOT NULL,
    comment_id integer,
    user_id integer,
    user_string character varying(80),
    text text,
    title character varying(256),
    date timestamp without time zone
);


ALTER TABLE public.comment_revision OWNER TO wikijump;

--
-- Name: comment_revision_revision_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.comment_revision_revision_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.comment_revision_revision_id_seq OWNER TO wikijump;

--
-- Name: comment_revision_revision_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.comment_revision_revision_id_seq OWNED BY public.comment_revision.revision_id;


--
-- Name: contact; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.contact (
    contact_id integer NOT NULL,
    user_id integer,
    target_user_id integer
);


ALTER TABLE public.contact OWNER TO wikijump;

--
-- Name: contact_contact_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.contact_contact_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.contact_contact_id_seq OWNER TO wikijump;

--
-- Name: contact_contact_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.contact_contact_id_seq OWNED BY public.contact.contact_id;


--
-- Name: domain_redirect; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.domain_redirect (
    redirect_id integer NOT NULL,
    site_id integer,
    url character varying(80)
);


ALTER TABLE public.domain_redirect OWNER TO wikijump;

--
-- Name: domain_redirect_redirect_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.domain_redirect_redirect_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.domain_redirect_redirect_id_seq OWNER TO wikijump;

--
-- Name: domain_redirect_redirect_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.domain_redirect_redirect_id_seq OWNED BY public.domain_redirect.redirect_id;


--
-- Name: email_invitation; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.email_invitation (
    invitation_id integer NOT NULL,
    hash character varying(200),
    email character varying(128),
    name character varying(100),
    user_id integer,
    site_id integer,
    become_member boolean DEFAULT true,
    to_contacts boolean,
    message text,
    attempts integer DEFAULT 1,
    accepted boolean DEFAULT false,
    delivered boolean DEFAULT true,
    date timestamp without time zone
);


ALTER TABLE public.email_invitation OWNER TO wikijump;

--
-- Name: email_invitation_invitation_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.email_invitation_invitation_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.email_invitation_invitation_id_seq OWNER TO wikijump;

--
-- Name: email_invitation_invitation_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.email_invitation_invitation_id_seq OWNED BY public.email_invitation.invitation_id;


--
-- Name: file; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.file (
    file_id integer NOT NULL,
    page_id integer,
    site_id integer,
    filename character varying(100),
    mimetype character varying(100),
    description character varying(200),
    description_short character varying(200),
    comment character varying(400),
    size integer,
    date_added timestamp without time zone,
    user_id integer,
    user_string character varying(80),
    has_resized boolean DEFAULT false
);


ALTER TABLE public.file OWNER TO wikijump;

--
-- Name: file_file_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.file_file_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.file_file_id_seq OWNER TO wikijump;

--
-- Name: file_file_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.file_file_id_seq OWNED BY public.file.file_id;


--
-- Name: files_event; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.files_event (
    file_event_id integer NOT NULL,
    filename character varying(100),
    date timestamp without time zone,
    user_id integer,
    user_string character varying(80),
    action character varying(80),
    action_extra character varying(80)
);


ALTER TABLE public.files_event OWNER TO wikijump;

--
-- Name: files_event_file_event_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.files_event_file_event_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.files_event_file_event_id_seq OWNER TO wikijump;

--
-- Name: files_event_file_event_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.files_event_file_event_id_seq OWNED BY public.files_event.file_event_id;


--
-- Name: form_submission_key; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.form_submission_key (
    key_id character varying(90) NOT NULL,
    date_submitted timestamp without time zone
);


ALTER TABLE public.form_submission_key OWNER TO wikijump;

--
-- Name: forum_category; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.forum_category (
    category_id integer NOT NULL,
    group_id integer,
    name character varying(80),
    description text,
    number_posts integer DEFAULT 0,
    number_threads integer DEFAULT 0,
    last_post_id integer,
    permissions_default boolean DEFAULT true,
    permissions character varying(200),
    max_nest_level integer,
    sort_index integer DEFAULT 0,
    site_id integer,
    per_page_discussion boolean DEFAULT false
);


ALTER TABLE public.forum_category OWNER TO wikijump;

--
-- Name: forum_category_category_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.forum_category_category_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.forum_category_category_id_seq OWNER TO wikijump;

--
-- Name: forum_category_category_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.forum_category_category_id_seq OWNED BY public.forum_category.category_id;


--
-- Name: forum_group; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.forum_group (
    group_id integer NOT NULL,
    name character varying(80),
    description text,
    sort_index integer DEFAULT 0,
    site_id integer,
    visible boolean DEFAULT true
);


ALTER TABLE public.forum_group OWNER TO wikijump;

--
-- Name: forum_group_group_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.forum_group_group_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.forum_group_group_id_seq OWNER TO wikijump;

--
-- Name: forum_group_group_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.forum_group_group_id_seq OWNED BY public.forum_group.group_id;


--
-- Name: forum_post; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.forum_post (
    post_id integer NOT NULL,
    thread_id integer,
    parent_id integer,
    user_id integer,
    user_string character varying(80),
    title character varying(256),
    text text,
    date_posted timestamp without time zone,
    site_id integer,
    revision_number integer DEFAULT 0,
    revision_id integer,
    date_last_edited timestamp without time zone,
    edited_user_id integer,
    edited_user_string character varying(80)
);


ALTER TABLE public.forum_post OWNER TO wikijump;

--
-- Name: forum_post_post_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.forum_post_post_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.forum_post_post_id_seq OWNER TO wikijump;

--
-- Name: forum_post_post_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.forum_post_post_id_seq OWNED BY public.forum_post.post_id;


--
-- Name: forum_post_revision; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.forum_post_revision (
    revision_id integer NOT NULL,
    post_id integer,
    user_id integer,
    user_string character varying(80),
    text text,
    title character varying(256),
    date timestamp without time zone
);


ALTER TABLE public.forum_post_revision OWNER TO wikijump;

--
-- Name: forum_post_revision_revision_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.forum_post_revision_revision_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.forum_post_revision_revision_id_seq OWNER TO wikijump;

--
-- Name: forum_post_revision_revision_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.forum_post_revision_revision_id_seq OWNED BY public.forum_post_revision.revision_id;


--
-- Name: forum_settings; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.forum_settings (
    site_id integer NOT NULL,
    permissions character varying(200),
    per_page_discussion boolean DEFAULT false,
    max_nest_level integer DEFAULT 0
);


ALTER TABLE public.forum_settings OWNER TO wikijump;

--
-- Name: forum_thread; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.forum_thread (
    thread_id integer NOT NULL,
    user_id integer,
    user_string character varying(80),
    category_id integer,
    title character varying(256),
    description character varying(1000),
    number_posts integer DEFAULT 1,
    date_started timestamp without time zone,
    site_id integer,
    last_post_id integer,
    page_id integer,
    sticky boolean DEFAULT false,
    blocked boolean DEFAULT false
);


ALTER TABLE public.forum_thread OWNER TO wikijump;

--
-- Name: forum_thread_thread_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.forum_thread_thread_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.forum_thread_thread_id_seq OWNER TO wikijump;

--
-- Name: forum_thread_thread_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.forum_thread_thread_id_seq OWNED BY public.forum_thread.thread_id;


--
-- Name: front_forum_feed; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.front_forum_feed (
    feed_id integer NOT NULL,
    page_id integer,
    title character varying(90),
    label character varying(90),
    description character varying(256),
    categories character varying(100),
    parmhash character varying(100),
    site_id integer
);


ALTER TABLE public.front_forum_feed OWNER TO wikijump;

--
-- Name: front_forum_feed_feed_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.front_forum_feed_feed_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.front_forum_feed_feed_id_seq OWNER TO wikijump;

--
-- Name: front_forum_feed_feed_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.front_forum_feed_feed_id_seq OWNED BY public.front_forum_feed.feed_id;


--
-- Name: fts_entry; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.fts_entry (
    fts_id integer NOT NULL,
    page_id integer,
    title character varying(256),
    unix_name character varying(100),
    thread_id integer,
    site_id integer,
    text text,
    vector tsvector
);


ALTER TABLE public.fts_entry OWNER TO wikijump;

--
-- Name: fts_entry_fts_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.fts_entry_fts_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.fts_entry_fts_id_seq OWNER TO wikijump;

--
-- Name: fts_entry_fts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.fts_entry_fts_id_seq OWNED BY public.fts_entry.fts_id;


--
-- Name: global_ip_block; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.global_ip_block (
    block_id integer NOT NULL,
    address inet,
    flag_proxy boolean DEFAULT false,
    reason text,
    flag_total boolean DEFAULT false,
    date_blocked timestamp without time zone
);


ALTER TABLE public.global_ip_block OWNER TO wikijump;

--
-- Name: global_ip_block_block_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.global_ip_block_block_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.global_ip_block_block_id_seq OWNER TO wikijump;

--
-- Name: global_ip_block_block_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.global_ip_block_block_id_seq OWNED BY public.global_ip_block.block_id;


--
-- Name: global_user_block; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.global_user_block (
    block_id integer NOT NULL,
    site_id integer,
    user_id integer,
    reason text,
    date_blocked timestamp without time zone
);


ALTER TABLE public.global_user_block OWNER TO wikijump;

--
-- Name: global_user_block_block_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.global_user_block_block_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.global_user_block_block_id_seq OWNER TO wikijump;

--
-- Name: global_user_block_block_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.global_user_block_block_id_seq OWNED BY public.global_user_block.block_id;


--
-- Name: ip_block; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.ip_block (
    block_id integer NOT NULL,
    site_id integer,
    ip inet,
    flag_proxy boolean DEFAULT false,
    reason text,
    date_blocked timestamp without time zone
);


ALTER TABLE public.ip_block OWNER TO wikijump;

--
-- Name: ip_block_block_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.ip_block_block_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.ip_block_block_id_seq OWNER TO wikijump;

--
-- Name: ip_block_block_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.ip_block_block_id_seq OWNED BY public.ip_block.block_id;


--
-- Name: license; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.license (
    license_id integer NOT NULL,
    name character varying(100),
    description text,
    sort integer DEFAULT 0
);


ALTER TABLE public.license OWNER TO wikijump;

--
-- Name: license_license_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.license_license_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.license_license_id_seq OWNER TO wikijump;

--
-- Name: license_license_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.license_license_id_seq OWNED BY public.license.license_id;


--
-- Name: log_event; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.log_event (
    event_id bigint NOT NULL,
    date timestamp without time zone,
    user_id integer,
    ip inet,
    proxy inet,
    type character varying(256),
    site_id integer,
    page_id integer,
    revision_id integer,
    thread_id integer,
    post_id integer,
    user_agent character varying(512),
    text text
);


ALTER TABLE public.log_event OWNER TO wikijump;

--
-- Name: log_event_event_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.log_event_event_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.log_event_event_id_seq OWNER TO wikijump;

--
-- Name: log_event_event_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.log_event_event_id_seq OWNED BY public.log_event.event_id;


--
-- Name: member; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.member (
    member_id integer NOT NULL,
    site_id integer,
    user_id integer,
    date_joined timestamp without time zone,
    allow_newsletter boolean DEFAULT true
);


ALTER TABLE public.member OWNER TO wikijump;

--
-- Name: member_application; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.member_application (
    application_id integer NOT NULL,
    site_id integer,
    user_id integer,
    status character varying(20) DEFAULT 'pending'::character varying,
    date timestamp without time zone,
    comment text,
    reply text
);


ALTER TABLE public.member_application OWNER TO wikijump;

--
-- Name: member_application_application_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.member_application_application_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.member_application_application_id_seq OWNER TO wikijump;

--
-- Name: member_application_application_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.member_application_application_id_seq OWNED BY public.member_application.application_id;


--
-- Name: member_invitation; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.member_invitation (
    invitation_id integer NOT NULL,
    site_id integer,
    user_id integer,
    by_user_id integer,
    date timestamp without time zone,
    body text
);


ALTER TABLE public.member_invitation OWNER TO wikijump;

--
-- Name: member_invitation_invitation_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.member_invitation_invitation_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.member_invitation_invitation_id_seq OWNER TO wikijump;

--
-- Name: member_invitation_invitation_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.member_invitation_invitation_id_seq OWNED BY public.member_invitation.invitation_id;


--
-- Name: member_member_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.member_member_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.member_member_id_seq OWNER TO wikijump;

--
-- Name: member_member_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.member_member_id_seq OWNED BY public.member.member_id;


--
-- Name: membership_link; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.membership_link (
    link_id integer NOT NULL,
    site_id integer,
    by_user_id integer,
    user_id integer,
    date timestamp without time zone,
    type character varying(20)
);


ALTER TABLE public.membership_link OWNER TO wikijump;

--
-- Name: membership_link_link_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.membership_link_link_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.membership_link_link_id_seq OWNER TO wikijump;

--
-- Name: membership_link_link_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.membership_link_link_id_seq OWNED BY public.membership_link.link_id;


--
-- Name: moderator; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.moderator (
    moderator_id integer NOT NULL,
    site_id integer,
    user_id integer,
    permissions character(10)
);


ALTER TABLE public.moderator OWNER TO wikijump;

--
-- Name: moderator_moderator_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.moderator_moderator_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.moderator_moderator_id_seq OWNER TO wikijump;

--
-- Name: moderator_moderator_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.moderator_moderator_id_seq OWNED BY public.moderator.moderator_id;


--
-- Name: notification; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.notification (
    notification_id integer NOT NULL,
    user_id integer,
    body text,
    type character varying(50),
    viewed boolean DEFAULT false,
    date timestamp without time zone,
    extra bytea,
    notify_online boolean DEFAULT true,
    notify_feed boolean DEFAULT false,
    notify_email boolean DEFAULT true
);


ALTER TABLE public.notification OWNER TO wikijump;

--
-- Name: notification_notification_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.notification_notification_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.notification_notification_id_seq OWNER TO wikijump;

--
-- Name: notification_notification_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.notification_notification_id_seq OWNED BY public.notification.notification_id;


--
-- Name: openid_entry; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.openid_entry (
    openid_id integer NOT NULL,
    site_id integer,
    page_id integer,
    type character varying(10),
    user_id integer,
    url character varying(100),
    server_url character varying(100)
);


ALTER TABLE public.openid_entry OWNER TO wikijump;

--
-- Name: openid_entry_openid_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.openid_entry_openid_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.openid_entry_openid_id_seq OWNER TO wikijump;

--
-- Name: openid_entry_openid_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.openid_entry_openid_id_seq OWNED BY public.openid_entry.openid_id;


--
-- Name: ozone_group; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.ozone_group (
    group_id integer NOT NULL,
    parent_group_id integer,
    name character varying(50),
    description text
);


ALTER TABLE public.ozone_group OWNER TO wikijump;

--
-- Name: ozone_group_group_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.ozone_group_group_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.ozone_group_group_id_seq OWNER TO wikijump;

--
-- Name: ozone_group_group_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.ozone_group_group_id_seq OWNED BY public.ozone_group.group_id;


--
-- Name: ozone_group_permission_modifier; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.ozone_group_permission_modifier (
    group_permission_id integer NOT NULL,
    group_id character varying(20),
    permission_id character varying(20),
    modifier integer
);


ALTER TABLE public.ozone_group_permission_modifier OWNER TO wikijump;

--
-- Name: ozone_group_permission_modifier_group_permission_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.ozone_group_permission_modifier_group_permission_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.ozone_group_permission_modifier_group_permission_id_seq OWNER TO wikijump;

--
-- Name: ozone_group_permission_modifier_group_permission_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.ozone_group_permission_modifier_group_permission_id_seq OWNED BY public.ozone_group_permission_modifier.group_permission_id;


--
-- Name: ozone_lock; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.ozone_lock (
    key character varying(100) NOT NULL
);


ALTER TABLE public.ozone_lock OWNER TO wikijump;

--
-- Name: ozone_permission; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.ozone_permission (
    permission_id integer NOT NULL,
    name character varying(50),
    description text
);


ALTER TABLE public.ozone_permission OWNER TO wikijump;

--
-- Name: ozone_permission_permission_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.ozone_permission_permission_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.ozone_permission_permission_id_seq OWNER TO wikijump;

--
-- Name: ozone_permission_permission_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.ozone_permission_permission_id_seq OWNED BY public.ozone_permission.permission_id;


--
-- Name: ozone_session; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.ozone_session (
    session_id character varying(60) NOT NULL,
    started timestamp without time zone,
    last_accessed timestamp without time zone,
    ip_address character varying(90),
    check_ip boolean DEFAULT false,
    infinite boolean DEFAULT false,
    user_id integer,
    serialized_datablock bytea,
    ip_address_ssl character varying(90),
    ua_hash character varying(256)
);


ALTER TABLE public.ozone_session OWNER TO wikijump;

--
-- Name: ozone_user; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.ozone_user (
    user_id integer NOT NULL,
    name character varying(99),
    nick_name character varying(70),
    password character varying(255),
    email character varying(99),
    unix_name character varying(99),
    last_login timestamp without time zone,
    registered_date timestamp without time zone,
    super_admin boolean DEFAULT false,
    super_moderator boolean DEFAULT false,
    language character varying(10) DEFAULT 'en'::character varying
);


ALTER TABLE public.ozone_user OWNER TO wikijump;

--
-- Name: ozone_user_group_relation; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.ozone_user_group_relation (
    user_group_id integer NOT NULL,
    user_id integer,
    group_id integer
);


ALTER TABLE public.ozone_user_group_relation OWNER TO wikijump;

--
-- Name: ozone_user_group_relation_user_group_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.ozone_user_group_relation_user_group_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.ozone_user_group_relation_user_group_id_seq OWNER TO wikijump;

--
-- Name: ozone_user_group_relation_user_group_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.ozone_user_group_relation_user_group_id_seq OWNED BY public.ozone_user_group_relation.user_group_id;


--
-- Name: ozone_user_permission_modifier; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.ozone_user_permission_modifier (
    user_permission_id integer NOT NULL,
    user_id integer,
    permission_id character varying(20),
    modifier integer
);


ALTER TABLE public.ozone_user_permission_modifier OWNER TO wikijump;

--
-- Name: ozone_user_permission_modifier_user_permission_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.ozone_user_permission_modifier_user_permission_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.ozone_user_permission_modifier_user_permission_id_seq OWNER TO wikijump;

--
-- Name: ozone_user_permission_modifier_user_permission_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.ozone_user_permission_modifier_user_permission_id_seq OWNED BY public.ozone_user_permission_modifier.user_permission_id;


--
-- Name: ozone_user_user_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.ozone_user_user_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.ozone_user_user_id_seq OWNER TO wikijump;

--
-- Name: ozone_user_user_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.ozone_user_user_id_seq OWNED BY public.ozone_user.user_id;


--
-- Name: page; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.page (
    page_id integer NOT NULL,
    site_id integer,
    category_id integer,
    parent_page_id integer,
    revision_id integer,
    source_id integer,
    metadata_id integer,
    revision_number integer DEFAULT 0,
    title character varying(256),
    unix_name character varying(256),
    date_created timestamp without time zone,
    date_last_edited timestamp without time zone,
    last_edit_user_id integer,
    last_edit_user_string character varying(80),
    thread_id integer,
    owner_user_id integer,
    blocked boolean DEFAULT false,
    rate integer DEFAULT 0
);


ALTER TABLE public.page OWNER TO wikijump;

--
-- Name: page_abuse_flag; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.page_abuse_flag (
    flag_id integer NOT NULL,
    user_id integer,
    site_id integer,
    path character varying(100),
    site_valid boolean DEFAULT true,
    global_valid boolean DEFAULT true
);


ALTER TABLE public.page_abuse_flag OWNER TO wikijump;

--
-- Name: page_abuse_flag_flag_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.page_abuse_flag_flag_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.page_abuse_flag_flag_id_seq OWNER TO wikijump;

--
-- Name: page_abuse_flag_flag_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.page_abuse_flag_flag_id_seq OWNED BY public.page_abuse_flag.flag_id;


--
-- Name: page_compiled; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.page_compiled (
    page_id integer NOT NULL,
    text text,
    date_compiled timestamp without time zone
);


ALTER TABLE public.page_compiled OWNER TO wikijump;

--
-- Name: page_edit_lock; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.page_edit_lock (
    lock_id integer NOT NULL,
    page_id integer,
    mode character varying(10) DEFAULT 'page'::character varying,
    section_id integer,
    range_start integer,
    range_end integer,
    page_unix_name character varying(100),
    user_id integer,
    user_string character varying(80),
    session_id character varying(60),
    date_started timestamp without time zone,
    date_last_accessed timestamp without time zone,
    secret character varying(100),
    site_id integer
);


ALTER TABLE public.page_edit_lock OWNER TO wikijump;

--
-- Name: page_edit_lock_lock_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.page_edit_lock_lock_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.page_edit_lock_lock_id_seq OWNER TO wikijump;

--
-- Name: page_edit_lock_lock_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.page_edit_lock_lock_id_seq OWNED BY public.page_edit_lock.lock_id;


--
-- Name: page_external_link; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.page_external_link (
    link_id integer NOT NULL,
    site_id integer,
    page_id integer,
    to_url character varying(512),
    pinged boolean DEFAULT false,
    ping_status character varying(256),
    date timestamp without time zone
);


ALTER TABLE public.page_external_link OWNER TO wikijump;

--
-- Name: page_external_link_link_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.page_external_link_link_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.page_external_link_link_id_seq OWNER TO wikijump;

--
-- Name: page_external_link_link_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.page_external_link_link_id_seq OWNED BY public.page_external_link.link_id;


--
-- Name: page_inclusion; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.page_inclusion (
    inclusion_id integer NOT NULL,
    including_page_id integer,
    included_page_id integer,
    included_page_name character varying(128),
    site_id integer
);


ALTER TABLE public.page_inclusion OWNER TO wikijump;

--
-- Name: page_inclusion_inclusion_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.page_inclusion_inclusion_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.page_inclusion_inclusion_id_seq OWNER TO wikijump;

--
-- Name: page_inclusion_inclusion_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.page_inclusion_inclusion_id_seq OWNED BY public.page_inclusion.inclusion_id;


--
-- Name: page_link; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.page_link (
    link_id integer NOT NULL,
    from_page_id integer,
    to_page_id integer,
    to_page_name character varying(128),
    site_id integer
);


ALTER TABLE public.page_link OWNER TO wikijump;

--
-- Name: page_link_link_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.page_link_link_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.page_link_link_id_seq OWNER TO wikijump;

--
-- Name: page_link_link_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.page_link_link_id_seq OWNED BY public.page_link.link_id;


--
-- Name: page_metadata; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.page_metadata (
    metadata_id integer NOT NULL,
    parent_page_id integer,
    title character varying(256),
    unix_name character varying(80),
    owner_user_id integer
);


ALTER TABLE public.page_metadata OWNER TO wikijump;

--
-- Name: page_metadata_metadata_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.page_metadata_metadata_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.page_metadata_metadata_id_seq OWNER TO wikijump;

--
-- Name: page_metadata_metadata_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.page_metadata_metadata_id_seq OWNED BY public.page_metadata.metadata_id;


--
-- Name: page_page_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.page_page_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.page_page_id_seq OWNER TO wikijump;

--
-- Name: page_page_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.page_page_id_seq OWNED BY public.page.page_id;


--
-- Name: page_rate_vote; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.page_rate_vote (
    rate_id integer NOT NULL,
    user_id integer,
    page_id integer,
    rate integer DEFAULT 1,
    date timestamp without time zone
);


ALTER TABLE public.page_rate_vote OWNER TO wikijump;

--
-- Name: page_rate_vote_rate_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.page_rate_vote_rate_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.page_rate_vote_rate_id_seq OWNER TO wikijump;

--
-- Name: page_rate_vote_rate_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.page_rate_vote_rate_id_seq OWNED BY public.page_rate_vote.rate_id;


--
-- Name: page_revision; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.page_revision (
    revision_id integer NOT NULL,
    page_id integer,
    source_id integer,
    metadata_id integer,
    flags character varying(100),
    flag_text boolean DEFAULT false,
    flag_title boolean DEFAULT false,
    flag_file boolean DEFAULT false,
    flag_rename boolean DEFAULT false,
    flag_meta boolean DEFAULT false,
    flag_new boolean DEFAULT false,
    since_full_source integer DEFAULT 0,
    diff_source boolean DEFAULT false,
    revision_number integer DEFAULT 0,
    date_last_edited timestamp without time zone,
    user_id integer,
    user_string character varying(80),
    comments text,
    flag_new_site boolean DEFAULT false,
    site_id integer
);


ALTER TABLE public.page_revision OWNER TO wikijump;

--
-- Name: page_revision_revision_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.page_revision_revision_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.page_revision_revision_id_seq OWNER TO wikijump;

--
-- Name: page_revision_revision_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.page_revision_revision_id_seq OWNED BY public.page_revision.revision_id;


--
-- Name: page_source; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.page_source (
    source_id integer NOT NULL,
    text text
);


ALTER TABLE public.page_source OWNER TO wikijump;

--
-- Name: page_source_source_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.page_source_source_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.page_source_source_id_seq OWNER TO wikijump;

--
-- Name: page_source_source_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.page_source_source_id_seq OWNED BY public.page_source.source_id;


--
-- Name: page_tag; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.page_tag (
    tag_id bigint NOT NULL,
    site_id integer,
    page_id integer,
    tag character varying(20)
);


ALTER TABLE public.page_tag OWNER TO wikijump;

--
-- Name: page_tag_tag_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.page_tag_tag_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.page_tag_tag_id_seq OWNER TO wikijump;

--
-- Name: page_tag_tag_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.page_tag_tag_id_seq OWNED BY public.page_tag.tag_id;


--
-- Name: petition_campaign; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.petition_campaign (
    campaign_id integer NOT NULL,
    site_id integer,
    name character varying(256),
    identifier character varying(256),
    active boolean DEFAULT true,
    number_signatures integer DEFAULT 0,
    deleted boolean DEFAULT false,
    collect_address boolean DEFAULT true,
    collect_city boolean DEFAULT true,
    collect_state boolean DEFAULT true,
    collect_zip boolean DEFAULT true,
    collect_country boolean DEFAULT true,
    collect_comments boolean DEFAULT true,
    show_city boolean DEFAULT true,
    show_state boolean DEFAULT true,
    show_zip boolean DEFAULT false,
    show_country boolean DEFAULT true,
    show_comments boolean DEFAULT false,
    thank_you_page character varying(256)
);


ALTER TABLE public.petition_campaign OWNER TO wikijump;

--
-- Name: petition_campaign_campaign_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.petition_campaign_campaign_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.petition_campaign_campaign_id_seq OWNER TO wikijump;

--
-- Name: petition_campaign_campaign_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.petition_campaign_campaign_id_seq OWNED BY public.petition_campaign.campaign_id;


--
-- Name: petition_signature; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.petition_signature (
    signature_id integer NOT NULL,
    campaign_id integer,
    first_name character varying(256),
    last_name character varying(256),
    address1 character varying(256),
    address2 character varying(256),
    zip character varying(256),
    city character varying(256),
    state character varying(256),
    country character varying(256),
    country_code character varying(8),
    comments text,
    email character varying(256),
    confirmed boolean DEFAULT false,
    confirmation_hash character varying(256),
    confirmation_url character varying(256),
    date timestamp without time zone
);


ALTER TABLE public.petition_signature OWNER TO wikijump;

--
-- Name: petition_signature_signature_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.petition_signature_signature_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.petition_signature_signature_id_seq OWNER TO wikijump;

--
-- Name: petition_signature_signature_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.petition_signature_signature_id_seq OWNED BY public.petition_signature.signature_id;


--
-- Name: private_message; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.private_message (
    message_id integer NOT NULL,
    from_user_id integer,
    to_user_id integer,
    subject character varying(256),
    body text,
    date timestamp without time zone,
    flag integer DEFAULT 0,
    flag_new boolean DEFAULT true
);


ALTER TABLE public.private_message OWNER TO wikijump;

--
-- Name: private_message_message_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.private_message_message_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.private_message_message_id_seq OWNER TO wikijump;

--
-- Name: private_message_message_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.private_message_message_id_seq OWNED BY public.private_message.message_id;


--
-- Name: private_user_block; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.private_user_block (
    block_id integer NOT NULL,
    user_id integer,
    blocked_user_id integer
);


ALTER TABLE public.private_user_block OWNER TO wikijump;

--
-- Name: private_user_block_block_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.private_user_block_block_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.private_user_block_block_id_seq OWNER TO wikijump;

--
-- Name: private_user_block_block_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.private_user_block_block_id_seq OWNED BY public.private_user_block.block_id;


--
-- Name: profile; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.profile (
    user_id integer NOT NULL,
    real_name character varying(70),
    gender character(1),
    birthday_day integer,
    birthday_month integer,
    birthday_year integer,
    about text,
    location character varying(70),
    website character varying(100),
    im_aim character varying(100),
    im_gadu_gadu character varying(100),
    im_google_talk character varying(100),
    im_icq character varying(100),
    im_jabber character varying(100),
    im_msn character varying(100),
    im_yahoo character varying(100),
    change_screen_name_count integer DEFAULT 0
);


ALTER TABLE public.profile OWNER TO wikijump;

--
-- Name: simpletodo_list; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.simpletodo_list (
    list_id integer NOT NULL,
    site_id integer,
    label character varying(256),
    title character varying(256),
    data text
);


ALTER TABLE public.simpletodo_list OWNER TO wikijump;

--
-- Name: simpletodo_list_list_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.simpletodo_list_list_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.simpletodo_list_list_id_seq OWNER TO wikijump;

--
-- Name: simpletodo_list_list_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.simpletodo_list_list_id_seq OWNED BY public.simpletodo_list.list_id;


--
-- Name: site; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.site (
    site_id integer NOT NULL,
    name character varying(50),
    subtitle character varying(60),
    unix_name character varying(80),
    description text,
    language character varying(10) DEFAULT 'en'::character varying,
    date_created timestamp without time zone,
    custom_domain character varying(60),
    visible boolean DEFAULT true,
    default_page character varying(80) DEFAULT 'start'::character varying,
    private boolean DEFAULT false,
    deleted boolean DEFAULT false
);


ALTER TABLE public.site OWNER TO wikijump;

--
-- Name: site_backup; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.site_backup (
    backup_id integer NOT NULL,
    site_id integer,
    status character varying(50),
    backup_source boolean DEFAULT true,
    backup_files boolean DEFAULT true,
    date timestamp without time zone,
    rand character varying(100)
);


ALTER TABLE public.site_backup OWNER TO wikijump;

--
-- Name: site_backup_backup_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.site_backup_backup_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.site_backup_backup_id_seq OWNER TO wikijump;

--
-- Name: site_backup_backup_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.site_backup_backup_id_seq OWNED BY public.site_backup.backup_id;


--
-- Name: site_settings; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.site_settings (
    site_id integer NOT NULL,
    allow_membership_by_apply boolean DEFAULT true,
    allow_membership_by_password boolean DEFAULT false,
    membership_password character varying(80),
    file_storage_size integer DEFAULT 314572800,
    use_ganalytics boolean DEFAULT false,
    private_landing_page character varying(80) DEFAULT 'system:join'::character varying,
    max_private_members integer DEFAULT 50,
    max_private_viewers integer DEFAULT 20,
    hide_navigation_unauthorized boolean DEFAULT true,
    ssl_mode character varying(20),
    openid_enabled boolean DEFAULT false,
    allow_members_invite boolean DEFAULT false,
    max_upload_file_size integer DEFAULT 10485760,
    enable_all_pingback_out boolean DEFAULT true
);


ALTER TABLE public.site_settings OWNER TO wikijump;

--
-- Name: site_site_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.site_site_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.site_site_id_seq OWNER TO wikijump;

--
-- Name: site_site_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.site_site_id_seq OWNED BY public.site.site_id;


--
-- Name: site_super_settings; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.site_super_settings (
    site_id integer NOT NULL,
    can_custom_domain boolean DEFAULT true
);


ALTER TABLE public.site_super_settings OWNER TO wikijump;

--
-- Name: site_tag; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.site_tag (
    tag_id integer NOT NULL,
    site_id integer,
    tag character varying(20)
);


ALTER TABLE public.site_tag OWNER TO wikijump;

--
-- Name: site_tag_tag_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.site_tag_tag_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.site_tag_tag_id_seq OWNER TO wikijump;

--
-- Name: site_tag_tag_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.site_tag_tag_id_seq OWNED BY public.site_tag.tag_id;


--
-- Name: site_viewer; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.site_viewer (
    viewer_id integer NOT NULL,
    site_id integer,
    user_id integer
);


ALTER TABLE public.site_viewer OWNER TO wikijump;

--
-- Name: site_viewer_viewer_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.site_viewer_viewer_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.site_viewer_viewer_id_seq OWNER TO wikijump;

--
-- Name: site_viewer_viewer_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.site_viewer_viewer_id_seq OWNED BY public.site_viewer.viewer_id;


--
-- Name: storage_item; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.storage_item (
    item_id character varying(256) NOT NULL,
    date timestamp without time zone,
    timeout integer,
    data bytea
);


ALTER TABLE public.storage_item OWNER TO wikijump;

--
-- Name: theme; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.theme (
    theme_id integer NOT NULL,
    name character varying(100),
    unix_name character varying(100),
    abstract boolean DEFAULT false,
    extends_theme_id integer,
    variant_of_theme_id integer,
    custom boolean DEFAULT false,
    site_id integer,
    use_side_bar boolean DEFAULT true,
    use_top_bar boolean DEFAULT true,
    sort_index integer DEFAULT 0,
    sync_page_name character varying(100),
    revision_number integer DEFAULT 0
);


ALTER TABLE public.theme OWNER TO wikijump;

--
-- Name: theme_preview; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.theme_preview (
    theme_id integer NOT NULL,
    body text
);


ALTER TABLE public.theme_preview OWNER TO wikijump;

--
-- Name: theme_theme_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.theme_theme_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.theme_theme_id_seq OWNER TO wikijump;

--
-- Name: theme_theme_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.theme_theme_id_seq OWNED BY public.theme.theme_id;


--
-- Name: ucookie; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.ucookie (
    ucookie_id character varying(100) NOT NULL,
    site_id integer,
    session_id character varying(60),
    date_granted timestamp without time zone
);


ALTER TABLE public.ucookie OWNER TO wikijump;

--
-- Name: unique_string_broker; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.unique_string_broker (
    last_index integer
);


ALTER TABLE public.unique_string_broker OWNER TO wikijump;

--
-- Name: user_abuse_flag; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.user_abuse_flag (
    flag_id integer NOT NULL,
    user_id integer,
    target_user_id integer,
    site_id integer,
    site_valid boolean DEFAULT true,
    global_valid boolean DEFAULT true
);


ALTER TABLE public.user_abuse_flag OWNER TO wikijump;

--
-- Name: user_abuse_flag_flag_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.user_abuse_flag_flag_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.user_abuse_flag_flag_id_seq OWNER TO wikijump;

--
-- Name: user_abuse_flag_flag_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.user_abuse_flag_flag_id_seq OWNED BY public.user_abuse_flag.flag_id;


--
-- Name: user_block; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.user_block (
    block_id integer NOT NULL,
    site_id integer,
    user_id integer,
    reason text,
    date_blocked timestamp without time zone
);


ALTER TABLE public.user_block OWNER TO wikijump;

--
-- Name: user_block_block_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.user_block_block_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.user_block_block_id_seq OWNER TO wikijump;

--
-- Name: user_block_block_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.user_block_block_id_seq OWNED BY public.user_block.block_id;


--
-- Name: user_karma; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.user_karma (
    user_id integer NOT NULL,
    points integer DEFAULT 0,
    level integer DEFAULT 0
);


ALTER TABLE public.user_karma OWNER TO wikijump;

--
-- Name: user_settings; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.user_settings (
    user_id integer NOT NULL,
    receive_invitations boolean DEFAULT true,
    receive_pm character(5) DEFAULT 'a'::bpchar,
    notify_online character varying(512) DEFAULT '*'::character varying,
    notify_feed character varying(512) DEFAULT '*'::character varying,
    notify_email character varying(512),
    receive_newsletter boolean DEFAULT true,
    receive_digest boolean DEFAULT true,
    allow_site_newsletters_default boolean DEFAULT true,
    max_sites_admin integer DEFAULT 3
);


ALTER TABLE public.user_settings OWNER TO wikijump;

--
-- Name: watched_forum_thread; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.watched_forum_thread (
    watched_id integer NOT NULL,
    user_id integer,
    thread_id integer
);


ALTER TABLE public.watched_forum_thread OWNER TO wikijump;

--
-- Name: watched_forum_thread_watched_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.watched_forum_thread_watched_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.watched_forum_thread_watched_id_seq OWNER TO wikijump;

--
-- Name: watched_forum_thread_watched_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.watched_forum_thread_watched_id_seq OWNED BY public.watched_forum_thread.watched_id;


--
-- Name: watched_page; Type: TABLE; Schema: public; Owner: wikijump
--

CREATE TABLE public.watched_page (
    watched_id integer NOT NULL,
    user_id integer,
    page_id integer
);


ALTER TABLE public.watched_page OWNER TO wikijump;

--
-- Name: watched_page_watched_id_seq; Type: SEQUENCE; Schema: public; Owner: wikijump
--

CREATE SEQUENCE public.watched_page_watched_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.watched_page_watched_id_seq OWNER TO wikijump;

--
-- Name: watched_page_watched_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: wikijump
--

ALTER SEQUENCE public.watched_page_watched_id_seq OWNED BY public.watched_page.watched_id;


--
-- Name: admin admin_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.admin ALTER COLUMN admin_id SET DEFAULT nextval('public.admin_admin_id_seq'::regclass);


--
-- Name: admin_notification notification_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.admin_notification ALTER COLUMN notification_id SET DEFAULT nextval('public.admin_notification_notification_id_seq'::regclass);


--
-- Name: anonymous_abuse_flag flag_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.anonymous_abuse_flag ALTER COLUMN flag_id SET DEFAULT nextval('public.anonymous_abuse_flag_flag_id_seq'::regclass);


--
-- Name: category category_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.category ALTER COLUMN category_id SET DEFAULT nextval('public.category_category_id_seq'::regclass);


--
-- Name: category_template category_template_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.category_template ALTER COLUMN category_template_id SET DEFAULT nextval('public.category_template_category_template_id_seq'::regclass);


--
-- Name: comment comment_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.comment ALTER COLUMN comment_id SET DEFAULT nextval('public.comment_comment_id_seq'::regclass);


--
-- Name: comment_revision revision_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.comment_revision ALTER COLUMN revision_id SET DEFAULT nextval('public.comment_revision_revision_id_seq'::regclass);


--
-- Name: contact contact_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.contact ALTER COLUMN contact_id SET DEFAULT nextval('public.contact_contact_id_seq'::regclass);


--
-- Name: domain_redirect redirect_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.domain_redirect ALTER COLUMN redirect_id SET DEFAULT nextval('public.domain_redirect_redirect_id_seq'::regclass);


--
-- Name: email_invitation invitation_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.email_invitation ALTER COLUMN invitation_id SET DEFAULT nextval('public.email_invitation_invitation_id_seq'::regclass);


--
-- Name: file file_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.file ALTER COLUMN file_id SET DEFAULT nextval('public.file_file_id_seq'::regclass);


--
-- Name: files_event file_event_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.files_event ALTER COLUMN file_event_id SET DEFAULT nextval('public.files_event_file_event_id_seq'::regclass);


--
-- Name: forum_category category_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_category ALTER COLUMN category_id SET DEFAULT nextval('public.forum_category_category_id_seq'::regclass);


--
-- Name: forum_group group_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_group ALTER COLUMN group_id SET DEFAULT nextval('public.forum_group_group_id_seq'::regclass);


--
-- Name: forum_post post_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_post ALTER COLUMN post_id SET DEFAULT nextval('public.forum_post_post_id_seq'::regclass);


--
-- Name: forum_post_revision revision_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_post_revision ALTER COLUMN revision_id SET DEFAULT nextval('public.forum_post_revision_revision_id_seq'::regclass);


--
-- Name: forum_thread thread_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_thread ALTER COLUMN thread_id SET DEFAULT nextval('public.forum_thread_thread_id_seq'::regclass);


--
-- Name: front_forum_feed feed_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.front_forum_feed ALTER COLUMN feed_id SET DEFAULT nextval('public.front_forum_feed_feed_id_seq'::regclass);


--
-- Name: fts_entry fts_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.fts_entry ALTER COLUMN fts_id SET DEFAULT nextval('public.fts_entry_fts_id_seq'::regclass);


--
-- Name: global_ip_block block_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.global_ip_block ALTER COLUMN block_id SET DEFAULT nextval('public.global_ip_block_block_id_seq'::regclass);


--
-- Name: global_user_block block_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.global_user_block ALTER COLUMN block_id SET DEFAULT nextval('public.global_user_block_block_id_seq'::regclass);


--
-- Name: ip_block block_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ip_block ALTER COLUMN block_id SET DEFAULT nextval('public.ip_block_block_id_seq'::regclass);


--
-- Name: license license_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.license ALTER COLUMN license_id SET DEFAULT nextval('public.license_license_id_seq'::regclass);


--
-- Name: log_event event_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.log_event ALTER COLUMN event_id SET DEFAULT nextval('public.log_event_event_id_seq'::regclass);


--
-- Name: member member_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.member ALTER COLUMN member_id SET DEFAULT nextval('public.member_member_id_seq'::regclass);


--
-- Name: member_application application_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.member_application ALTER COLUMN application_id SET DEFAULT nextval('public.member_application_application_id_seq'::regclass);


--
-- Name: member_invitation invitation_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.member_invitation ALTER COLUMN invitation_id SET DEFAULT nextval('public.member_invitation_invitation_id_seq'::regclass);


--
-- Name: membership_link link_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.membership_link ALTER COLUMN link_id SET DEFAULT nextval('public.membership_link_link_id_seq'::regclass);


--
-- Name: moderator moderator_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.moderator ALTER COLUMN moderator_id SET DEFAULT nextval('public.moderator_moderator_id_seq'::regclass);


--
-- Name: notification notification_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.notification ALTER COLUMN notification_id SET DEFAULT nextval('public.notification_notification_id_seq'::regclass);


--
-- Name: openid_entry openid_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.openid_entry ALTER COLUMN openid_id SET DEFAULT nextval('public.openid_entry_openid_id_seq'::regclass);


--
-- Name: ozone_group group_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_group ALTER COLUMN group_id SET DEFAULT nextval('public.ozone_group_group_id_seq'::regclass);


--
-- Name: ozone_group_permission_modifier group_permission_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_group_permission_modifier ALTER COLUMN group_permission_id SET DEFAULT nextval('public.ozone_group_permission_modifier_group_permission_id_seq'::regclass);


--
-- Name: ozone_permission permission_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_permission ALTER COLUMN permission_id SET DEFAULT nextval('public.ozone_permission_permission_id_seq'::regclass);


--
-- Name: ozone_user user_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_user ALTER COLUMN user_id SET DEFAULT nextval('public.ozone_user_user_id_seq'::regclass);


--
-- Name: ozone_user_group_relation user_group_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_user_group_relation ALTER COLUMN user_group_id SET DEFAULT nextval('public.ozone_user_group_relation_user_group_id_seq'::regclass);


--
-- Name: ozone_user_permission_modifier user_permission_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_user_permission_modifier ALTER COLUMN user_permission_id SET DEFAULT nextval('public.ozone_user_permission_modifier_user_permission_id_seq'::regclass);


--
-- Name: page page_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page ALTER COLUMN page_id SET DEFAULT nextval('public.page_page_id_seq'::regclass);


--
-- Name: page_abuse_flag flag_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_abuse_flag ALTER COLUMN flag_id SET DEFAULT nextval('public.page_abuse_flag_flag_id_seq'::regclass);


--
-- Name: page_edit_lock lock_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_edit_lock ALTER COLUMN lock_id SET DEFAULT nextval('public.page_edit_lock_lock_id_seq'::regclass);


--
-- Name: page_external_link link_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_external_link ALTER COLUMN link_id SET DEFAULT nextval('public.page_external_link_link_id_seq'::regclass);


--
-- Name: page_inclusion inclusion_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_inclusion ALTER COLUMN inclusion_id SET DEFAULT nextval('public.page_inclusion_inclusion_id_seq'::regclass);


--
-- Name: page_link link_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_link ALTER COLUMN link_id SET DEFAULT nextval('public.page_link_link_id_seq'::regclass);


--
-- Name: page_metadata metadata_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_metadata ALTER COLUMN metadata_id SET DEFAULT nextval('public.page_metadata_metadata_id_seq'::regclass);


--
-- Name: page_rate_vote rate_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_rate_vote ALTER COLUMN rate_id SET DEFAULT nextval('public.page_rate_vote_rate_id_seq'::regclass);


--
-- Name: page_revision revision_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_revision ALTER COLUMN revision_id SET DEFAULT nextval('public.page_revision_revision_id_seq'::regclass);


--
-- Name: page_source source_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_source ALTER COLUMN source_id SET DEFAULT nextval('public.page_source_source_id_seq'::regclass);


--
-- Name: page_tag tag_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_tag ALTER COLUMN tag_id SET DEFAULT nextval('public.page_tag_tag_id_seq'::regclass);


--
-- Name: petition_campaign campaign_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.petition_campaign ALTER COLUMN campaign_id SET DEFAULT nextval('public.petition_campaign_campaign_id_seq'::regclass);


--
-- Name: petition_signature signature_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.petition_signature ALTER COLUMN signature_id SET DEFAULT nextval('public.petition_signature_signature_id_seq'::regclass);


--
-- Name: private_message message_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.private_message ALTER COLUMN message_id SET DEFAULT nextval('public.private_message_message_id_seq'::regclass);


--
-- Name: private_user_block block_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.private_user_block ALTER COLUMN block_id SET DEFAULT nextval('public.private_user_block_block_id_seq'::regclass);


--
-- Name: simpletodo_list list_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.simpletodo_list ALTER COLUMN list_id SET DEFAULT nextval('public.simpletodo_list_list_id_seq'::regclass);


--
-- Name: site site_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site ALTER COLUMN site_id SET DEFAULT nextval('public.site_site_id_seq'::regclass);


--
-- Name: site_backup backup_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site_backup ALTER COLUMN backup_id SET DEFAULT nextval('public.site_backup_backup_id_seq'::regclass);


--
-- Name: site_tag tag_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site_tag ALTER COLUMN tag_id SET DEFAULT nextval('public.site_tag_tag_id_seq'::regclass);


--
-- Name: site_viewer viewer_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site_viewer ALTER COLUMN viewer_id SET DEFAULT nextval('public.site_viewer_viewer_id_seq'::regclass);


--
-- Name: theme theme_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.theme ALTER COLUMN theme_id SET DEFAULT nextval('public.theme_theme_id_seq'::regclass);


--
-- Name: user_abuse_flag flag_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.user_abuse_flag ALTER COLUMN flag_id SET DEFAULT nextval('public.user_abuse_flag_flag_id_seq'::regclass);


--
-- Name: user_block block_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.user_block ALTER COLUMN block_id SET DEFAULT nextval('public.user_block_block_id_seq'::regclass);


--
-- Name: watched_forum_thread watched_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.watched_forum_thread ALTER COLUMN watched_id SET DEFAULT nextval('public.watched_forum_thread_watched_id_seq'::regclass);


--
-- Name: watched_page watched_id; Type: DEFAULT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.watched_page ALTER COLUMN watched_id SET DEFAULT nextval('public.watched_page_watched_id_seq'::regclass);


--
-- Data for Name: admin; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.admin (admin_id, site_id, user_id, founder) FROM stdin;
\.


--
-- Data for Name: admin_notification; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.admin_notification (notification_id, site_id, body, type, viewed, date, extra, notify_online, notify_feed, notify_email) FROM stdin;
\.


--
-- Data for Name: anonymous_abuse_flag; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.anonymous_abuse_flag (flag_id, user_id, address, proxy, site_id, site_valid, global_valid) FROM stdin;
\.


--
-- Data for Name: category; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, enable_pingback_out, enable_pingback_in) FROM stdin;
6	2	nav	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f
7	3	_default	t	20	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	f	1	\N	f	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f
9	3	admin	f	21	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f
11	3	nav	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f
14	2	search	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f
15	1	nav	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f
2	2	_default	t	20	f	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	f	1	\N	f	nav:top	nav:side	\N	f	t	\N	\N	\N	t	f
13	2	admin	f	21	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f
17	2	forum	t	20	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f
12	2	system	t	20	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f
1	1	_default	t	20	f	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	f	1	\N	f	nav:top	nav:side	\N	f	t	\N	\N	\N	t	f
4	1	account	f	21	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f
3	1	admin	f	21	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f
16	1	search	t	20	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f
5	1	user	f	21	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f
8	3	profile	f	20	f	e:o;c:;m:;d:;a:;r:;z:;o:o	t	1	\N	f	nav:top	nav:profile-side	\N	\N	t	\N	\N	\N	t	f
18	2	profile	t	20	t	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f
19	1	system-all	t	20	t	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f
20	1	system	t	20	t	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f
21	1	auth	t	20	f	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	f	1	\N	f	nav:top	nav:side	\N	f	t	\N	\N	\N	t	f
\.


--
-- Data for Name: category_template; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.category_template (category_template_id, source) FROM stdin;
\.


--
-- Data for Name: comment; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.comment (comment_id, page_id, parent_id, user_id, user_string, title, text, date_posted, site_id, revision_number, revision_id, date_last_edited, edited_user_id, edited_user_string) FROM stdin;
\.


--
-- Data for Name: comment_revision; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.comment_revision (revision_id, comment_id, user_id, user_string, text, title, date) FROM stdin;
\.


--
-- Data for Name: contact; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.contact (contact_id, user_id, target_user_id) FROM stdin;
\.


--
-- Data for Name: domain_redirect; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.domain_redirect (redirect_id, site_id, url) FROM stdin;
\.


--
-- Data for Name: email_invitation; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.email_invitation (invitation_id, hash, email, name, user_id, site_id, become_member, to_contacts, message, attempts, accepted, delivered, date) FROM stdin;
\.


--
-- Data for Name: file; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.file (file_id, page_id, site_id, filename, mimetype, description, description_short, comment, size, date_added, user_id, user_string, has_resized) FROM stdin;
\.


--
-- Data for Name: files_event; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.files_event (file_event_id, filename, date, user_id, user_string, action, action_extra) FROM stdin;
\.


--
-- Data for Name: form_submission_key; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.form_submission_key (key_id, date_submitted) FROM stdin;
\.


--
-- Data for Name: forum_category; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.forum_category (category_id, group_id, name, description, number_posts, number_threads, last_post_id, permissions_default, permissions, max_nest_level, sort_index, site_id, per_page_discussion) FROM stdin;
\.


--
-- Data for Name: forum_group; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.forum_group (group_id, name, description, sort_index, site_id, visible) FROM stdin;
\.


--
-- Data for Name: forum_post; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.forum_post (post_id, thread_id, parent_id, user_id, user_string, title, text, date_posted, site_id, revision_number, revision_id, date_last_edited, edited_user_id, edited_user_string) FROM stdin;
\.


--
-- Data for Name: forum_post_revision; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.forum_post_revision (revision_id, post_id, user_id, user_string, text, title, date) FROM stdin;
\.


--
-- Data for Name: forum_settings; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.forum_settings (site_id, permissions, per_page_discussion, max_nest_level) FROM stdin;
2	t:m;p:m;e:o;s:	f	2
\.


--
-- Data for Name: forum_thread; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.forum_thread (thread_id, user_id, user_string, category_id, title, description, number_posts, date_started, site_id, last_post_id, page_id, sticky, blocked) FROM stdin;
\.


--
-- Data for Name: front_forum_feed; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.front_forum_feed (feed_id, page_id, title, label, description, categories, parmhash, site_id) FROM stdin;
\.


--
-- Data for Name: fts_entry; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) FROM stdin;
32	32	Top	nav:top	\N	2	\n\n\nexample menu\n\nsubmenu\n\n\ncontact\n\n	'nav':2C 'top':1C,3C 'menu':5 'exampl':4 'contact':7 'submenu':6
33	33	Template	profile:template	\N	2	\n\nProfile has not been created (yet).\n	'yet':9 'creat':8 'profil':2C,4 'templat':1C,3C
34	5	Side	nav:side	\N	2	\n\n\nWelcome page\n\n\nWhat is a Wiki Site?\nHow to edit pages?\n\n\nHow to join this site?\nSite members\n\n\nRecent changes\nList all pages\nPage Tags\n\n\nSite Manager\n\nPage tags\n\n\nAdd a new page\n\n\nedit this panel\n	'add':33 'nav':2C 'new':35 'tag':28,32 'edit':13,37 'join':17 'list':24 'page':5,14,26,27,31,36 'side':1C,3C 'site':10,19,20,29 'wiki':9 'chang':23 'manag':30 'panel':39 'member':21 'recent':22 'welcom':4
37	36	Congratulations, welcome to your new wiki!	start	\N	2	\n\nIf this is your first site\nThen there are some things you need to know:\n\nYou can configure all security and other settings online, using the Site Manager. When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself. Check out the Permissions section.\nYour Wikidot site has two menus, one at the side called 'nav:side', and one at the top called 'nav:top'. These are Wikidot pages, and you can edit them like any page.\nTo edit a page, go to the page and click the Edit button at the bottom. You can change everything in the main area of your page. The Wikidot system is easy to learn and powerful.\nYou can attach images and other files to any page, then display them and link to them in the page.\nEvery Wikidot page has a history of edits, and you can undo anything. So feel secure, and experiment.\nTo start a forum on your site, see the Site Manager » Forum.\nThe license for this Wikidot site has been set to Creative Commons Attribution-Share Alike 3.0 License. If you want to change this, use the Site Manager.\nIf you want to learn more, make sure you visit the Documentation section at www.wikidot.org\n\nMore information about the Wikidot project can be found at www.wikidot.org.\n	'go':104 '3.0':203 'nav':78,86 'new':5C 'one':73,81 'see':181 'set':30,195 'top':84,87 'two':71 'use':32,211 '�':185 'alik':202 'area':123 'call':77,85 'easi':131 'edit':95,101,111,163 'feel':170 'file':142 'help':42 'imag':139 'know':22 'like':60,97 'link':150 'main':122 'make':57,221 'need':20 'page':91,99,103,107,126,145,155,158 'side':76,79 'site':13,34,45,53,69,180,183,192,213 'sure':222 'undo':167 'want':207,217 'wiki':6C 'anyth':168 'build':43 'chang':118,209 'check':62 'click':109 'everi':156 'first':12 'forum':177,186 'found':238 'invit':38 'learn':133,219 'manag':35,54,184,214 'menus':72 'onlin':31 'peopl':40 'power':135 'secur':27,171 'share':201 'start':7C,175 'thing':18 'visit':224 'access':50 'attach':138 'bottom':115 'button':112 'common':198 'experi':173 'inform':231 'licens':188,204 'system':129 'unless':55 'welcom':2C 'creativ':197 'display':147 'everyth':119 'histori':161 'permiss':65 'project':235 'section':66,227 'wikidot':68,90,128,157,191,234 'attribut':200 'configur':25 'document':226 'administr':59 'congratul':1C 'www.wikidot.org':229,240 'attribution-shar':199
45	13	How To Edit Pages - Quickstart	how-to-edit-pages	\N	2	\n\nIf you are allowed to edit pages in this Site, simply click on edit button at the bottom of the page. This will open an editor with a toolbar pallette with options.\nTo create a link to a new page, use syntax: [[[new page name]]] or [[[new page name | text to display]]]. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!\nAlthough creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit Documentation pages (at wikidot.org) to learn more.\n	'lot':95 'new':49,53,57,80 'use':51 'easi':91 'edit':3C,9C,16,24,83,88 'link':46,65 'name':55,59 'open':34 'page':4C,10C,17,31,50,54,58,73,81,89,106 'site':20,102 'text':60 'allow':14,99 'click':22 'color':71 'creat':44,78,86,100 'exist':76 'learn':110 'pleas':103 'power':101 'visit':104 'bottom':28 'button':25 'differ':70 'editor':36 'follow':63 'option':42,97 'simpli':21 'syntax':52 'display':62 'pallett':40 'toolbar':39 'although':85 'document':105 'quickstart':5C 'wikidot.org':108 'how-to-edit-pag':6C
39	37	List of all wikis	system-all:all-sites	\N	1	\n\nBelow is the list of public visible Wikis hosted at this service:\n\n	'host':19 'list':1C,14 'site':10C 'wiki':4C,18 'public':16 'servic':22 'system':6C 'visibl':17 'all-sit':8C 'system-al':5C
40	38	List wikis by tags	system-all:sites-by-tags	\N	1	\n\n\n\n	'tag':4C,11C 'list':1C 'site':9C 'wiki':2C 'system':6C 'system-al':5C 'sites-by-tag':8C
42	40	Activity across all wikis	system-all:activity	\N	1	\n\n\n\n\nRecent edits (all wikis)\n\n\n\nTop Sites\n\n\nTop Forums\n\n\nNew users\n\n\nSome statistics\n\n\n\n\n	'new':17 'top':13,15 'edit':10 'site':14 'user':18 'wiki':4C,12 'activ':1C,8C 'forum':16 'across':2C 'recent':9 'system':6C 'statist':20 'system-al':5C
53	1	Manage	admin:manage	\N	1	\n\n\n	'admin':2C 'manag':1C,3C
54	2	You	account:you	\N	1	\n\n\n	'account':1C
55	3	Get a new wiki	new-site	\N	1	\n\nUse this simple form to create a new wiki.\nTo admins: you can customize this page by simply clicking "edit" at the bottom of the page.\n\n	'get':1C 'new':3C,6C,15 'use':8 'edit':27 'form':11 'page':23,33 'site':7C 'wiki':4C,16 'admin':18 'click':26 'creat':13 'simpl':10 'bottom':30 'custom':21 'simpli':25 'new-sit':5C
56	4	Info	user:info	\N	1	\n\n\n	'info':1C,3C 'user':2C
57	24	Search All Wikis	search:all	\N	1	\n\n\n	'wiki':3C 'search':1C,4C
58	25	Search This Wiki	search:site	\N	1	\n\n\n	'site':5C 'wiki':3C 'search':1C,4C
59	26	Search Users	search:users	\N	1	\n\nTo look for someone, please enter:\n\nemail address of a person you are looking for (this will look for exact match)\nany part of the screen name or realname (lists all Users matching the query)\n\n\n	'list':34 'look':6,18,22 'name':31 'part':27 'user':2C,4C,36 'email':11 'enter':10 'exact':24 'match':25,37 'pleas':9 'queri':39 'person':15 'screen':30 'search':1C,3C 'someon':8 'address':12 'realnam':33
38	22	Side	nav:side	\N	1	\n\n\nWelcome page\n\n\nWhat is a Wiki?\nHow to edit pages?\nGet a new wiki!\n\nAll wikis\n\nRecent activity\nAll wikis\nWikis by tags\nSearch\n\nThis wiki\n\nHow to join this site?\nSite members\n\n\nRecent changes\nList all pages\nPage Tags\n\n\nSite Manager\n\nPage tags\n\n\nAdd a new page\n\n\nedit this panel\n	'add':48 'get':14 'nav':2C 'new':16,50 'tag':26,43,47 'edit':12,52 'join':32 'list':39 'page':5,13,41,42,46,51 'side':1C,3C 'site':34,35,44 'wiki':9,17,19,23,24,29 'activ':21 'chang':38 'manag':45 'panel':54 'member':36 'recent':20,37 'search':27 'welcom':4
41	39	Search	system-all:search	\N	1	\n\n\nSearch all Wikis\nPerform a search through all public and visible wikis.\n\n\n\nSearch users\nTo look for someone, please enter:\n\nemail address of a person you are looking for (this will look for exact match)\nany part of the screen name or realname (lists all Users matching the query)\n\n\n\n	'list':49 'look':21,33,37 'name':46 'part':42 'user':19,51 'wiki':8,17 'email':26 'enter':25 'exact':39 'match':40,52 'pleas':24 'queri':54 'person':30 'public':14 'screen':45 'search':1C,5C,6,11,18 'someon':23 'system':3C 'visibl':16 'address':27 'perform':9 'realnam':48 'system-al':2C
47	43	Wiki Members	system:members	\N	1	\n\nMembers:\n\n\nModerators\n\n\nAdmins\n\n	'wiki':1C 'admin':7 'moder':6 'member':2C,4C,5 'system':3C
49	45	Recent changes	system:recent-changes	\N	1	\n\n\n	'chang':2C,6C 'recent':1C,5C 'system':3C 'recent-chang':4C
52	48	Page Tags	system:page-tags	\N	1	\n\n\n\n\n	'tag':2C,6C 'page':1C,5C 'system':3C 'page-tag':4C
43	23	Welcome to your new Wikidot installation!	start	\N	1	\n\nCongratulations, you have successfully installed Wikidot software on your computer!\nWhat to do next\nCustomize this wiki\nWikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!\n\nYou can configure all security and other settings online, using the Site Manager. When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself. Check out the Permissions section.\nYour Wikidot site has two menus, one at the side called 'nav:side', and one at the top called 'nav:top'. These are Wikidot pages, and you can edit them like any page.\nTo edit a page, go to the page and click the Edit button at the bottom. You can change everything in the main area of your page. The Wikidot system is easy to learn and powerful.\nYou can attach images and other files to any page, then display them and link to them in the page.\nEvery Wikidot page has a history of edits, and you can undo anything. So feel secure, and experiment.\nTo start a forum on your site, see the Site Manager » Forum.\nThe license for this Wikidot site has been set to Creative Commons Attribution-Share Alike 3.0 License. If you want to change this, use the Site Manager.\nIf you want to learn more, make sure you visit the Documentation section at www.wikidot.org\n\nCustomize default template\nDefault initial template for other wikis is located at template-en. If someone creates a new wiki, this one is cloned to the new address. A good thing to do is to go to template-en and customize it.\nCreate more templates\nSimply create wikis with unix names starting with "template-" (e.g. "template-pl", "template-blog") and your users will be able to choose which wiki they want to start with.\nVisit Wikidot.org\nGo to www.wikidot.org — home of the Wikidot software — for extra documentation, howtos, tips and support.\n\nMore information about the Wikidot project can be found at www.wikidot.org.\nSearch all wikis\n\n\nSearch users\n\n	'en':264,290 'go':125,286,330 'pl':309 '3.0':223 'abl':318 'e.g':306 'nav':99,107 'new':4C,269,277 'one':33,94,102,272 'see':202 'set':51,215 'tip':342 'top':105,108 'two':92 'use':53,231 'alik':222 'area':144 'blog':312 'call':98,106 'easi':152 'edit':116,122,132,184 'feel':191 'file':163 'good':280 'help':63 'home':333 'imag':160 'like':81,118 'link':171 'main':40,143 'make':78,241 'name':302 'next':21 'page':112,120,124,128,147,166,176,179 'side':97,100 'site':30,55,66,74,90,201,204,212,233 'sure':242 'undo':188 'unix':301 'user':315,360 'want':227,237,324 'wiki':24,29,41,258,270,299,322,358 'anyth':189 'build':64 'chang':139,229 'check':83 'choos':320 'click':130 'clone':274 'creat':267,294,298 'everi':177 'extra':339 'forum':198,206 'found':353 'howto':341 'initi':254 'invit':59 'learn':154,239 'locat':260 'manag':56,75,205,234 'menus':93 'onlin':52 'peopl':61 'power':156 'right':34 'secur':48,192 'sever':28 'share':221 'start':7C,196,303,326 'thing':281 'visit':244,328 'access':71 'attach':159 'bottom':136 'button':133 'common':218 'comput':17 'custom':22,42,250,292 'experi':194 'inform':346 'instal':6C,12 'licens':208,224 'search':356,359 'simpli':297 'someon':266 'system':150 'unless':76 'welcom':1C 'address':278 'consist':26 'creativ':217 'default':251,253 'display':168 'everyth':140 'histori':182 'permiss':86 'project':350 'section':87,247 'softwar':14,337 'success':11 'support':344 'templat':252,255,263,289,296,305,308,311 'wikidot':5C,13,25,89,111,149,178,211,336,349 'attribut':220 'configur':46 'document':246,340 'administr':80 'congratul':8 'template-en':262,288 'template-pl':307 'wikidot.org':329 'template-blog':310 'www.wikidot.org':249,332,355 'attribution-shar':219
44	41	What Is A Wiki	what-is-a-wiki	\N	1	\n\nAccording to Wikipedia, the world largest wiki site:\n\nA Wiki ([ˈwiː.kiː] &lt;wee-kee&gt; or [ˈwɪ.kiː] &lt;wick-ey&gt;) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.\n\nAnd that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.\n	'ey':30 'add':40 'kee':24 'use':74 'wee':23 'edit':44 'farm':62 'file':79 'kiː':21,27 'part':59 'site':17,66 'tool':70 'type':33 'user':38 'wick':29 'wiki':4C,9C,16,19,64 'allow':37 'chang':46 'great':69 'quick':50 'remov':41 'world':14 'ˈwɪ':26 'accord':10 'easili':52 'upload':78 'websit':35 'wee-ke':22 'ˈwiː':20 'content':48,77 'largest':15 'publish':76 'wick-ey':28 'collabor':82 'communic':80 'otherwis':43 'wikipedia':12 'what-is-a-wiki':5C
48	44	How to join this wiki?	system:join	\N	1	\n\n\nPlease change this page according to your policy (configure first using Site Manager) and remove this note.\n\nWho can join?\nYou can write here who can become a member of this site.\nJoin!\nSo you want to become a member of this site? Tell us why and apply now!\n\n\nOr, if you already know a "secret password", go for it!\n\n	'go':65 'us':52 'use':18 'join':3C,7C,27,40 'know':61 'note':24 'page':11 'site':19,39,50 'tell':51 'want':43 'wiki':5C 'appli':55 'becom':34,45 'chang':9 'first':17 'manag':20 'pleas':8 'remov':22 'write':30 'accord':12 'member':36,47 'polici':15 'secret':63 'system':6C 'alreadi':60 'configur':16 'password':64
46	42	How To Edit Pages	how-to-edit-pages	\N	1	\n\nIf you are allowed to edit pages in this Site, simply click on edit button at the bottom of the page. This will open an editor with a toolbar pallette with options.\nTo create a link to a new page, use syntax: [[[new page name]]] or [[[new page name | text to display]]]. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!\nAlthough creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit Documentation pages (at wikidot.org) to learn more.\n	'lot':94 'new':48,52,56,79 'use':50 'easi':90 'edit':3C,8C,15,23,82,87 'link':45,64 'name':54,58 'open':33 'page':4C,9C,16,30,49,53,57,72,80,88,105 'site':19,101 'text':59 'allow':13,98 'click':21 'color':70 'creat':43,77,85,99 'exist':75 'learn':109 'pleas':102 'power':100 'visit':103 'bottom':27 'button':24 'differ':69 'editor':35 'follow':62 'option':41,96 'simpli':20 'syntax':51 'display':61 'pallett':39 'toolbar':38 'although':84 'document':104 'wikidot.org':107 'how-to-edit-pag':5C
50	46	List all pages	system:list-all-pages	\N	1	\n\n\n	'list':1C,6C 'page':3C,8C 'system':4C 'list-all-pag':5C
51	47	Page Tags List	system:page-tags-list	\N	1	\n\n\n	'tag':2C,7C 'list':3C,8C 'page':1C,6C 'system':4C 'page-tags-list':5C
60	49	Log in	auth:login	\N	1	\n\n\n	'log':1C 'auth':2C 'login':3C
61	50	Create account - step 1	auth:newaccount	\N	1	\n\n\n	'1':4C 'auth':5C 'step':3C 'creat':1C 'account':2C 'newaccount':6C
62	51	Create account - step 2	auth:newaccount2	\N	1	\n\n\n	'2':4C 'auth':5C 'step':3C 'creat':1C 'account':2C 'newaccount2':6C
63	52	Create account - step 3	auth:newaccount3	\N	1	\n\n\n	'3':4C 'auth':5C 'step':3C 'creat':1C 'account':2C 'newaccount3':6C
\.


--
-- Data for Name: global_ip_block; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.global_ip_block (block_id, address, flag_proxy, reason, flag_total, date_blocked) FROM stdin;
\.


--
-- Data for Name: global_user_block; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.global_user_block (block_id, site_id, user_id, reason, date_blocked) FROM stdin;
\.


--
-- Data for Name: ip_block; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.ip_block (block_id, site_id, ip, flag_proxy, reason, date_blocked) FROM stdin;
\.


--
-- Data for Name: license; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.license (license_id, name, description, sort) FROM stdin;
1	Creative Commons Attribution-ShareAlike 4.0 License (recommended)	%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-sa/4.0/">Creative Commons Attribution-ShareAlike 4.0 License</a>	1
2	Creative Commons Attribution 4.0 License	%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by/4.0/">Creative Commons Attribution 4.0 License</a>	2
3	Creative Commons Attribution-NoDerivs 4.0 License	%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-nd/4.0/">Creative Commons Attribution-NoDerivs 4.0 License</a>	3
4	Creative Commons Attribution-NonCommercial 4.0 License	%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-nc/4.0/">Creative Commons Attribution-NonCommercial 4.0 License</a>	4
5	Creative Commons Attribution-NonCommercial-ShareAlike 4.0 License	%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-nc-sa/4.0/">Creative Commons Attribution-NonCommercial-ShareAlike 4.0 License</a>	5
6	Creative Commons Attribution-NonCommercial-NoDerivs 4.0 License	%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-nc-nd/4.0/">Creative Commons Attribution-NonCommercial-NoDerivs 4.0 License</a>	6
7	Creative Commons Attribution-ShareAlike 3.0 License	%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-sa/3.0/">Creative Commons Attribution-ShareAlike 3.0 License</a>	7
8	Creative Commons Attribution 3.0 License	%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by/3.0/">Creative Commons Attribution 3.0 License</a>	8
9	Creative Commons Attribution-NoDerivs 3.0 License	%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-nd/3.0/">Creative Commons Attribution-NoDerivs 3.0 License</a>	9
10	Creative Commons Attribution-NonCommercial 3.0 License	%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-nc/3.0/">Creative Commons Attribution-NonCommercial 3.0 License</a>	10
11	Creative Commons Attribution-NonCommercial-ShareAlike 3.0 License	%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-nc-sa/3.0/">Creative Commons Attribution-NonCommercial-ShareAlike 3.0 License</a>	11
12	Creative Commons Attribution-NonCommercial-NoDerivs 3.0 License	%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-nc-nd/3.0/">Creative Commons Attribution-NonCommercial-NoDerivs 3.0 License</a>	12
13	CC0 (Public Domain)	%%UNLESS%%  \n<a rel="license" href="https://creativecommons.org/publicdomain/zero/1.0/">CC0 (Public Domain)</a>.	100
14	GNU Free Documentation License 1.3	%%UNLESS%%  \n<a rel="license" href="http://www.gnu.org/copyleft/fdl.html">GNU \nFree Documentation License</a>.	101
15	Standard copyright (not recommended)	\N	1000
\.


--
-- Data for Name: log_event; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.log_event (event_id, date, user_id, ip, proxy, type, site_id, page_id, revision_id, thread_id, post_id, user_agent, text) FROM stdin;
\.


--
-- Data for Name: member; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.member (member_id, site_id, user_id, date_joined, allow_newsletter) FROM stdin;
\.


--
-- Data for Name: member_application; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.member_application (application_id, site_id, user_id, status, date, comment, reply) FROM stdin;
\.


--
-- Data for Name: member_invitation; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.member_invitation (invitation_id, site_id, user_id, by_user_id, date, body) FROM stdin;
\.


--
-- Data for Name: membership_link; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.membership_link (link_id, site_id, by_user_id, user_id, date, type) FROM stdin;
\.


--
-- Data for Name: moderator; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.moderator (moderator_id, site_id, user_id, permissions) FROM stdin;
\.


--
-- Data for Name: notification; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.notification (notification_id, user_id, body, type, viewed, date, extra, notify_online, notify_feed, notify_email) FROM stdin;
\.


--
-- Data for Name: openid_entry; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.openid_entry (openid_id, site_id, page_id, type, user_id, url, server_url) FROM stdin;
\.


--
-- Data for Name: ozone_group; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.ozone_group (group_id, parent_group_id, name, description) FROM stdin;
\.


--
-- Data for Name: ozone_group_permission_modifier; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.ozone_group_permission_modifier (group_permission_id, group_id, permission_id, modifier) FROM stdin;
\.


--
-- Data for Name: ozone_lock; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.ozone_lock (key) FROM stdin;
\.


--
-- Data for Name: ozone_permission; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.ozone_permission (permission_id, name, description) FROM stdin;
\.


--
-- Data for Name: ozone_session; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.ozone_session (session_id, started, last_accessed, ip_address, check_ip, infinite, user_id, serialized_datablock, ip_address_ssl, ua_hash) FROM stdin;
\.


--
-- Data for Name: ozone_user; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.ozone_user (user_id, name, nick_name, password, email, unix_name, last_login, registered_date, super_admin, super_moderator, language) FROM stdin;
-1	Automatic	Automatic	\N	automatic@wikidot	automatic	\N	\N	f	f	en
0	Anonymous	Anonymous	\N	anonymous@wikidot	anonymous	\N	\N	f	f	en
1	Administrator	Admin	$2y$11$lqdZZxU8FYqV2LJNaCQTGeI4V9dW/.cPd.DXpx4geF99bhzzpqQMy	admin@wikidot	admin	\N	\N	t	f	en
\.


--
-- Data for Name: ozone_user_group_relation; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.ozone_user_group_relation (user_group_id, user_id, group_id) FROM stdin;
\.


--
-- Data for Name: ozone_user_permission_modifier; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.ozone_user_permission_modifier (user_permission_id, user_id, permission_id, modifier) FROM stdin;
\.


--
-- Data for Name: page; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) FROM stdin;
1	1	3	\N	1	1	1	0	\N	admin:manage	2008-01-24 12:16:34	2008-01-24 12:16:34	1	\N	\N	1	f	0
2	1	4	\N	2	2	2	0	\N	account:you	2008-01-24 12:22:02	2008-01-24 12:22:02	1	\N	\N	1	f	0
3	1	1	\N	3	3	3	0	Get a new wiki	new-site	2008-01-24 12:27:10	2008-01-24 12:27:10	1	\N	\N	1	f	0
4	1	5	\N	4	4	4	0	\N	user:info	2008-01-24 12:32:21	2008-01-24 12:32:21	1	\N	\N	1	f	0
5	2	6	\N	5	5	5	0	Side	nav:side	2008-01-25 00:35:20	2008-01-25 00:35:20	1	\N	\N	1	f	0
6	2	2	\N	6	6	6	0	What Is A Wiki Site	what-is-a-wiki-site	2008-01-25 00:45:30	2008-01-25 00:45:30	1	\N	\N	1	f	0
7	3	8	\N	7	7	7	0	Admin	profile:admin	2008-01-25 01:05:59	2008-01-25 01:05:59	1	\N	\N	1	f	0
8	3	9	\N	8	8	8	0	\N	admin:manage	2008-01-25 01:06:39	2008-01-25 01:06:39	1	\N	\N	1	f	0
10	3	11	\N	10	10	10	0	Profile Side	nav:profile-side	2008-01-25 01:09:41	2008-01-25 01:09:41	1	\N	\N	1	f	0
11	3	11	\N	12	12	11	1	Side	nav:side	2008-01-25 01:13:41	2008-01-25 01:14:31	1	\N	\N	1	f	0
12	3	7	\N	13	13	12	0	\N	start	2008-01-25 01:15:35	2008-01-25 01:15:35	1	\N	\N	1	f	0
14	2	12	\N	15	15	14	0	Join This Wiki	system:join	2008-01-29 00:56:59	2008-01-29 00:56:59	1	\N	\N	1	f	0
15	2	13	\N	16	16	15	0	\N	admin:manage	2008-01-29 00:57:39	2008-01-29 00:57:39	1	\N	\N	1	f	0
16	2	12	\N	17	17	16	0	Page Tags List	system:page-tags-list	2008-01-29 00:58:44	2008-01-29 00:58:44	1	\N	\N	1	f	0
17	2	12	\N	18	18	17	0	Recent Changes	system:recent-changes	2008-01-29 00:59:14	2008-01-29 00:59:14	1	\N	\N	1	f	0
18	2	12	\N	19	19	18	0	Members	system:members	2008-01-29 00:59:40	2008-01-29 00:59:40	1	\N	\N	1	f	0
19	2	14	\N	20	20	19	0	Wiki Search	search:site	2008-01-29 01:01:49	2008-01-29 01:01:49	1	\N	\N	1	f	0
20	2	12	\N	21	21	20	0	\N	system:page-tags	2008-01-29 01:03:43	2008-01-29 01:03:43	1	\N	\N	1	f	0
21	2	12	\N	22	22	21	0	List All Pages	system:list-all-pages	2008-01-29 01:04:52	2008-01-29 01:04:52	1	\N	\N	1	f	0
24	1	16	\N	25	25	24	0	Search All Wikis	search:all	2008-01-29 01:09:17	2008-01-29 01:09:17	1	\N	\N	1	f	0
25	1	16	\N	27	26	26	1	Search This Wiki	search:site	2008-01-29 01:34:40	2008-01-29 01:34:57	1	\N	\N	1	f	0
26	1	16	\N	30	29	27	1	Search Users	search:users	2008-01-29 01:36:56	2008-01-29 01:37:12	1	\N	\N	1	f	0
27	2	17	\N	31	30	28	0	Forum Categories	forum:start	2008-01-29 01:40:23	2008-01-29 01:40:23	1	\N	\N	1	f	0
28	2	17	\N	32	31	29	0	Forum Category	forum:category	2008-01-29 01:40:59	2008-01-29 01:40:59	1	\N	\N	1	f	0
29	2	17	\N	33	32	30	0	Forum Thread	forum:thread	2008-01-29 01:41:32	2008-01-29 01:41:32	1	\N	\N	1	f	0
30	2	17	\N	34	33	31	0	New Forum Thread	forum:new-thread	2008-01-29 01:42:10	2008-01-29 01:42:10	1	\N	\N	1	f	0
31	2	17	\N	35	34	32	0	Recent Forum Posts	forum:recent-posts	2008-01-29 01:42:42	2008-01-29 01:42:42	1	\N	\N	1	f	0
32	2	6	\N	36	35	33	0	Top	nav:top	2008-01-29 23:29:51	2008-01-29 23:29:51	1	\N	\N	1	f	0
33	2	18	\N	37	36	34	0	Template	profile:template	2008-01-29 23:30:18	2008-01-29 23:30:18	1	\N	\N	1	f	0
36	2	2	\N	40	39	37	0	Congratulations, welcome to your new wiki!	start	2008-01-30 08:43:22	2008-01-30 08:43:22	1	\N	\N	1	f	0
37	1	19	\N	42	41	38	0	List of all wikis	system-all:all-sites	2008-01-30 08:54:56	2008-01-30 08:54:56	1	\N	\N	1	f	0
38	1	19	\N	44	43	40	1	List wikis by tags	system-all:sites-by-tags	2008-01-30 08:55:33	2008-01-30 09:00:00	1	\N	\N	1	f	0
22	1	15	\N	45	44	22	2	Side	nav:side	2008-01-29 01:05:47	2008-01-30 09:01:50	1	\N	\N	1	f	0
39	1	19	\N	46	45	41	0	Search	system-all:search	2008-01-30 09:07:05	2008-01-30 09:07:05	1	\N	\N	1	f	0
40	1	19	\N	48	47	43	1	Activity across all wikis	system-all:activity	2008-01-30 09:16:38	2008-01-30 09:17:40	1	\N	\N	1	f	0
23	1	1	\N	50	49	44	3	Welcome to your new Wikidot installation!	start	2008-01-29 01:07:41	2008-01-30 16:08:02	1	\N	\N	1	f	0
41	1	1	\N	51	50	45	0	What Is A Wiki	what-is-a-wiki	2008-01-30 16:11:56	2008-01-30 16:11:56	1	\N	\N	1	f	0
13	2	2	\N	52	51	13	1	How To Edit Pages - Quickstart	how-to-edit-pages	2008-01-29 00:09:59	2008-01-30 16:12:40	1	\N	\N	1	f	0
42	1	1	\N	53	52	46	0	How To Edit Pages	how-to-edit-pages	2008-01-30 16:12:48	2008-01-30 16:12:48	1	\N	\N	1	f	0
43	1	20	\N	54	53	47	0	Wiki Members	system:members	2008-01-30 16:13:32	2008-01-30 16:13:32	1	\N	\N	1	f	0
44	1	20	\N	55	54	48	0	How to join this wiki?	system:join	2008-01-30 16:14:13	2008-01-30 16:14:13	1	\N	\N	1	f	0
45	1	20	\N	56	55	49	0	Recent changes	system:recent-changes	2008-01-30 16:14:41	2008-01-30 16:14:41	1	\N	\N	1	f	0
46	1	20	\N	57	56	50	0	List all pages	system:list-all-pages	2008-01-30 16:15:22	2008-01-30 16:15:22	1	\N	\N	1	f	0
47	1	20	\N	58	57	51	0	Page Tags List	system:page-tags-list	2008-01-30 16:15:56	2008-01-30 16:15:56	1	\N	\N	1	f	0
48	1	20	\N	59	58	52	0	Page Tags	system:page-tags	2008-01-30 16:16:22	2008-01-30 16:16:22	1	\N	\N	1	f	0
49	1	21	\N	60	59	53	0	Log in	auth:login	2008-08-19 16:25:58	2008-08-19 16:25:58	1	\N	\N	1	f	0
50	1	21	\N	61	60	54	0	Create account - step 1	auth:newaccount	2008-08-19 16:25:58	2008-08-19 16:25:58	1	\N	\N	1	f	0
51	1	21	\N	62	61	55	0	Create account - step 2	auth:newaccount2	2008-08-19 16:25:58	2008-08-19 16:25:58	1	\N	\N	1	f	0
52	1	21	\N	63	62	56	0	Create account - step 3	auth:newaccount3	2008-08-19 16:25:58	2008-08-19 16:25:58	1	\N	\N	1	f	0
\.


--
-- Data for Name: page_abuse_flag; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.page_abuse_flag (flag_id, user_id, site_id, path, site_valid, global_valid) FROM stdin;
\.


--
-- Data for Name: page_compiled; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.page_compiled (page_id, text, date_compiled) FROM stdin;
6	\n\n<p>According to <a href="http://en.wikipedia.org/wiki/Wiki">Wikipedia</a>, the world largest wiki site:</p>\n<blockquote>\n<p>A <em>Wiki</em> ([ˈwiː.kiː] &lt;wee-kee&gt; or [ˈwɪ.kiː] &lt;wick-ey&gt;) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.</p>\n</blockquote>\n<p>And that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.</p>\n	2008-01-25 00:45:30
7	\n\n<p>Admin of this Wikidot installation.</p>\n	2008-01-25 01:05:59
8	\n\nþmodule "managesite/ManageSiteModule"þ	2008-01-25 01:06:39
10	\n\n<p>The profiles site is used to host user profiles. Each <tt>profile:username</tt> page contains a user-editable text that is included in the user's profile page.</p>\n<p>If you are viewing your own profile content page, feel free to edit it. You are the only one allowed to edit this page.</p>\n	2008-01-25 01:09:41
12	\n\n<p>The purpose of this wiki is to store user profiles.</p>\n	2008-01-25 01:15:35
11	\n\n<p>The profiles site is used to host user profiles. Each <tt>profile:username</tt> page contains a user-editable text that is included in the user's profile page.</p>\n<ul>\n<li><a href="/start">Main page</a></li>\n<li><a href="/admin:manage">Manage this wiki</a></li>\n</ul>\n	2008-01-25 01:15:35
15	\n\nþmodule "managesite/ManageSiteModule"þ	2008-01-29 00:57:39
14	\n\n<div class="wiki-note">\n<p>Please change this page according to your policy (configure first using <a href="/admin:manage">Site Manager</a>) and remove this note.</p>\n</div>\n<h1 id="toc0"><span>Who can join?</span></h1>\n<p>You can write here who can become a member of this site.</p>\n<h1 id="toc1"><span>Join!</span></h1>\n<p>So you want to become a member of this site? Tell us why and apply now!</p>\nþmodule "membership/MembershipApplyModule"þ<br />\n<p>Or, if you already know a "secret password", go for it!</p>\nþmodule "membership/MembershipByPasswordModule"þ	2008-01-29 00:57:39
16	\n\nþmodule "wiki/pagestagcloud/PagesTagCloudModule" limit%3D%22200%22+target%3D%22system%3Apage-tags%22 þ	2008-01-29 00:58:44
17	\n\nþmodule "changes/SiteChangesModule"þ	2008-01-29 00:59:15
18	\n\n<h1 id="toc0"><span>Members:</span></h1>\nþmodule "membership/MembersListModule"þ\n<h1 id="toc1"><span>Moderators</span></h1>\nþmodule "membership/MembersListModule" group%3D%22moderators%22 þ\n<h1 id="toc2"><span>Admins</span></h1>\nþmodule "membership/MembersListModule" group%3D%22admins%22 þ	2008-01-29 00:59:40
19	\n\nþmodule "search/SearchModule"þ	2008-01-29 01:01:49
20	\n\n<div style="float:right; width: 50%;">þmodule "wiki/pagestagcloud/PagesTagCloudModule" limit%3D%22200%22+target%3D%22system%3Apage-tags%22 þ</div>\nþmodule "wiki/pagestagcloud/PagesListByTagModule"þ	2008-01-29 01:03:43
21	\n\nþmodule "list/WikiPagesModule" preview%3D%22true%22 þ	2008-01-29 01:04:52
27	\n\nþmodule "forum/ForumStartModule"þ	2008-01-29 01:40:24
28	\n\nþmodule "forum/ForumViewCategoryModule"þ	2008-01-29 01:40:59
29	\n\nþmodule "forum/ForumViewThreadModule"þ	2008-01-29 01:41:32
30	\n\nþmodule "forum/ForumNewThreadModule"þ	2008-01-29 01:42:10
31	\n\nþmodule "forum/ForumRecentPostsModule"þ	2008-01-29 01:42:42
32	\n\n<ul>\n<li><a href="javascript:;">example menu</a>\n<ul>\n<li><a class="newpage" href="/submenu">submenu</a></li>\n</ul>\n</li>\n<li><a class="newpage" href="/contact">contact</a></li>\n</ul>\n	2008-01-29 23:29:51
33	\n\n<p>Profile has not been created (yet).</p>\n	2008-01-29 23:30:18
5	\n\n<ul>\n<li><a href="/start">Welcome page</a></li>\n</ul>\n<ul>\n<li><a href="/what-is-a-wiki-site">What is a Wiki Site?</a></li>\n<li><a href="/how-to-edit-pages">How to edit pages?</a></li>\n</ul>\n<ul>\n<li><a href="/system:join">How to join this site?</a></li>\n<li><a href="/system:members">Site members</a></li>\n</ul>\n<ul>\n<li><a href="/system:recent-changes">Recent changes</a></li>\n<li><a href="/system:list-all-pages">List all pages</a></li>\n<li><a href="/system:page-tags-list">Page Tags</a></li>\n</ul>\n<ul>\n<li><a href="/admin:manage">Site Manager</a></li>\n</ul>\n<h2 id="toc0"><span>Page tags</span></h2>\nþmodule "wiki/pagestagcloud/PagesTagCloudModule" minFontSize%3D%2280%25%22+maxFontSize%3D%22200%25%22++maxColor%3D%228%2C8%2C64%22+minColor%3D%22100%2C100%2C128%22+target%3D%22system%3Apage-tags%22+limit%3D%2230%22 þ\n<h2 id="toc1"><span>Add a new page</span></h2>\nþmodule "misc/NewPageHelperModule" size%3D%2215%22+button%3D%22new+page%22 þ\n<p style="text-align: center;"><span style="font-size:80%;"><a href="/nav:side">edit this panel</a></span></p>\n	2008-01-30 08:39:25
36	\n\n<h2 id="toc0"><span>If this is your first site</span></h2>\n<p>Then there are some things you need to know:</p>\n<ul>\n<li>You can configure all security and other settings online, using the <a href="/admin:manage">Site Manager</a>. When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself. Check out the <em>Permissions</em> section.</li>\n<li>Your Wikidot site has two menus, <a href="/nav:side">one at the side</a> called '<tt>nav:side</tt>', and <a href="/nav:top">one at the top</a> called '<tt>nav:top</tt>'. These are Wikidot pages, and you can edit them like any page.</li>\n<li>To edit a page, go to the page and click the <strong>Edit</strong> button at the bottom. You can change everything in the main area of your page. The Wikidot system is <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, '_blank'); return false;">easy to learn and powerful</a>.</li>\n<li>You can attach images and other files to any page, then display them and link to them in the page.</li>\n<li>Every Wikidot page has a history of edits, and you can undo anything. So feel secure, and experiment.</li>\n<li>To start a forum on your site, see the <a href="/admin:manage">Site Manager</a> » <em>Forum</em>.</li>\n<li>The license for this Wikidot site has been set to <a href="http://creativecommons.org/licenses/by-sa/3.0/" onclick="window.open(this.href, '_blank'); return false;">Creative Commons Attribution-Share Alike 3.0 License</a>. If you want to change this, use the Site Manager.</li>\n<li>If you want to learn more, make sure you visit the <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, '_blank'); return false;">Documentation section at www.wikidot.org</a></li>\n</ul>\n<p>More information about the Wikidot project can be found at <a href="http://www.wikidot.org" onclick="window.open(this.href, '_blank'); return false;">www.wikidot.org</a>.</p>\n	2008-01-30 08:43:22
13	\n\n<p>If you are allowed to edit pages in this Site, simply click on <em>edit</em> button at the bottom of the page. This will open an editor with a toolbar pallette with options.</p>\n<p>To create a link to a new page, use syntax: <tt>[[[new page name]]]</tt> or <tt>[[[new page name | text to display]]]</tt>. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!</p>\n<p>Although creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, '_blank'); return false;">Documentation pages</a> (at wikidot.org) to learn more.</p>\n	2008-01-30 16:12:40
45	\n\nþmodule "changes/SiteChangesModule"þ	2008-08-19 16:25:59
1	\n\nþmodule "managesite/ManageSiteModule"þ	2008-08-19 16:25:58
2	\n\nþmodule "account/AccountModule"þ	2008-08-19 16:25:58
3	\n\n<p>Use this simple form to create a new wiki.</p>\n<p>To admins: you can customize this page by simply clicking "edit" at the bottom of the page.</p>\nþmodule "newsite/NewSiteModule"þ	2008-08-19 16:25:58
4	\n\nþmodule "userinfo/UserInfoModule"þ	2008-08-19 16:25:58
24	\n\nþmodule "search/SearchAllModule"þ	2008-08-19 16:25:58
25	\n\nþmodule "search/SearchModule"þ	2008-08-19 16:25:58
26	\n\n<p>To look for someone, please enter:</p>\n<ul>\n<li>email address of a person you are looking for (this will look for exact match)</li>\n<li>any part of the screen name or realname (lists all Users matching the query)</li>\n</ul>\nþmodule "search/UserSearchModule"þ	2008-08-19 16:25:58
37	\n\n<p>Below is the list of public visible Wikis hosted at this service:</p>\nþmodule "wiki/listallwikis/ListAllWikisModule"þ	2008-08-19 16:25:58
38	\n\nþmodule "wiki/sitestagcloud/SitesTagCloudModule" limit%3D%22100%22+target%3D%22system-all%3Asites-by-tags%22 þþmodule "wiki/sitestagcloud/SitesListByTagModule"þ	2008-08-19 16:25:58
22	\n\n<ul>\n<li><a href="/start">Welcome page</a></li>\n</ul>\n<ul>\n<li><a href="/what-is-a-wiki">What is a Wiki?</a></li>\n<li><a href="/how-to-edit-pages">How to edit pages?</a></li>\n<li><a href="/new-site">Get a new wiki!</a></li>\n</ul>\n<h1 id="toc0"><span>All wikis</span></h1>\n<ul>\n<li><a href="/system-all:activity">Recent activity</a></li>\n<li><a href="/system-all:all-sites">All wikis</a></li>\n<li><a href="/system-all:sites-by-tags">Wikis by tags</a></li>\n<li><a href="/system-all:search">Search</a></li>\n</ul>\n<h1 id="toc1"><span>This wiki</span></h1>\n<ul>\n<li><a href="/system:join">How to join this site?</a></li>\n<li><a href="/system:members">Site members</a></li>\n</ul>\n<ul>\n<li><a href="/system:recent-changes">Recent changes</a></li>\n<li><a href="/system:list-all-pages">List all pages</a></li>\n<li><a href="/system:page-tags-list">Page Tags</a></li>\n</ul>\n<ul>\n<li><a href="/admin:manage">Site Manager</a></li>\n</ul>\n<h2 id="toc2"><span>Page tags</span></h2>\nþmodule "wiki/pagestagcloud/PagesTagCloudModule" minFontSize%3D%2280%25%22+maxFontSize%3D%22200%25%22++maxColor%3D%228%2C8%2C64%22+minColor%3D%22100%2C100%2C128%22+target%3D%22system%3Apage-tags%22+limit%3D%2230%22 þ\n<h2 id="toc3"><span>Add a new page</span></h2>\nþmodule "misc/NewPageHelperModule" size%3D%2215%22+button%3D%22new+page%22 þ\n<p style="text-align: center;"><span style="font-size:80%;"><a href="/nav:side">edit this panel</a></span></p>\n	2008-08-19 16:25:58
39	\n\n<div style="text-align: center;">\n<h1 id="toc0"><span>Search all Wikis</span></h1>\n<p>Perform a search through all public and visible wikis.</p>\nþmodule "search/SearchAllModule"þ\n<hr />\n<h1 id="toc1"><span>Search users</span></h1>\n<p>To look for someone, please enter:</p>\n<ul>\n<li>email address of a person you are looking for (this will look for exact match)</li>\n<li>any part of the screen name or realname (lists all Users matching the query)</li>\n</ul>\nþmodule "search/UserSearchModule"þ</div>\n	2008-08-19 16:25:59
40	\n\n<table>\n<tr>\n<td style="width: 45%; padding-right: 2%; border-right: 1px solid #999; vertical-align:top;">\n<h2 id="toc0"><span>Recent edits (all wikis)</span></h2>\nþmodule "wiki/sitesactivity/RecentWPageRevisionsModule"þ</td>\n<td style="width: 45%; padding-left: 2%; vertical-align:top;">\n<h2 id="toc1"><span>Top Sites</span></h2>\nþmodule "wiki/sitesactivity/MostActiveSitesModule"þ\n<h2 id="toc2"><span>Top Forums</span></h2>\nþmodule "wiki/sitesactivity/MostActiveForumsModule"þ\n<h2 id="toc3"><span>New users</span></h2>\nþmodule "wiki/sitesactivity/NewWUsersModule"þ\n<h2 id="toc4"><span>Some statistics</span></h2>\nþmodule "wiki/sitesactivity/SomeGlobalStatsModule"þ</td>\n</tr>\n</table>\n	2008-08-19 16:25:59
23	\n\n<p>Congratulations, you have successfully installed Wikidot software on your computer!</p>\n<h1 id="toc0"><span>What to do next</span></h1>\n<h2 id="toc1"><span>Customize this wiki</span></h2>\n<p>Wikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!</p>\n<ul>\n<li>You can configure all security and other settings online, using the <a href="/admin:manage">Site Manager</a>. When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself. Check out the <em>Permissions</em> section.</li>\n<li>Your Wikidot site has two menus, <a href="/nav:side">one at the side</a> called '<tt>nav:side</tt>', and <a class="newpage" href="/nav:top">one at the top</a> called '<tt>nav:top</tt>'. These are Wikidot pages, and you can edit them like any page.</li>\n<li>To edit a page, go to the page and click the <strong>Edit</strong> button at the bottom. You can change everything in the main area of your page. The Wikidot system is <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, '_blank'); return false;">easy to learn and powerful</a>.</li>\n<li>You can attach images and other files to any page, then display them and link to them in the page.</li>\n<li>Every Wikidot page has a history of edits, and you can undo anything. So feel secure, and experiment.</li>\n<li>To start a forum on your site, see the <a href="/admin:manage">Site Manager</a> » <em>Forum</em>.</li>\n<li>The license for this Wikidot site has been set to <a href="http://creativecommons.org/licenses/by-sa/3.0/" onclick="window.open(this.href, '_blank'); return false;">Creative Commons Attribution-Share Alike 3.0 License</a>. If you want to change this, use the Site Manager.</li>\n<li>If you want to learn more, make sure you visit the <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, '_blank'); return false;">Documentation section at www.wikidot.org</a></li>\n</ul>\n<h2 id="toc2"><span>Customize default template</span></h2>\n<p>Default initial template for other wikis is located at <a href="http://template-en.wikidot1.dev/template-en">template-en</a>. If someone creates a new wiki, this one is cloned to the new address. A good thing to do is to go to <a href="http://template-en.wikidot1.dev/template-en">template-en</a> and customize it.</p>\n<h2 id="toc3"><span>Create more templates</span></h2>\n<p>Simply create wikis with unix names starting with "template-" (e.g. "template-pl", "template-blog") and your users will be able to choose which wiki they want to start with.</p>\n<h2 id="toc4"><span>Visit Wikidot.org</span></h2>\n<p>Go to <strong><a href="http://www.wikidot.org">www.wikidot.org</a></strong> — home of the Wikidot software — for extra documentation, howtos, tips and support.</p>\n<hr />\n<p>More information about the Wikidot project can be found at <a href="http://www.wikidot.org" onclick="window.open(this.href, '_blank'); return false;">www.wikidot.org</a>.</p>\n<h1 id="toc5"><span>Search all wikis</span></h1>\nþmodule "search/SearchAllModule"þ\n<h1 id="toc6"><span>Search users</span></h1>\nþmodule "search/UserSearchModule"þ	2008-08-19 16:25:59
41	\n\n<p>According to <a href="http://en.wikipedia.org/wiki/Wiki">Wikipedia</a>, the world largest wiki site:</p>\n<blockquote>\n<p>A <em>Wiki</em> ([ˈwiː.kiː] &lt;wee-kee&gt; or [ˈwɪ.kiː] &lt;wick-ey&gt;) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.</p>\n</blockquote>\n<p>And that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.</p>\n	2008-08-19 16:25:59
42	\n\n<p>If you are allowed to edit pages in this Site, simply click on <em>edit</em> button at the bottom of the page. This will open an editor with a toolbar pallette with options.</p>\n<p>To create a link to a new page, use syntax: <tt>[[[new page name]]]</tt> or <tt>[[[new page name | text to display]]]</tt>. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!</p>\n<p>Although creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, '_blank'); return false;">Documentation pages</a> (at wikidot.org) to learn more.</p>\n	2008-08-19 16:25:59
43	\n\n<h1 id="toc0"><span>Members:</span></h1>\nþmodule "membership/MembersListModule"þ\n<h1 id="toc1"><span>Moderators</span></h1>\nþmodule "membership/MembersListModule" group%3D%22moderators%22 þ\n<h1 id="toc2"><span>Admins</span></h1>\nþmodule "membership/MembersListModule" group%3D%22admins%22 þ	2008-08-19 16:25:59
44	\n\n<div class="wiki-note">\n<p>Please change this page according to your policy (configure first using <a href="/admin:manage">Site Manager</a>) and remove this note.</p>\n</div>\n<h1 id="toc0"><span>Who can join?</span></h1>\n<p>You can write here who can become a member of this site.</p>\n<h1 id="toc1"><span>Join!</span></h1>\n<p>So you want to become a member of this site? Tell us why and apply now!</p>\nþmodule "membership/MembershipApplyModule"þ\n<p>Or, if you already know a "secret password", go for it!</p>\nþmodule "membership/MembershipByPasswordModule"þ	2008-08-19 16:25:59
46	\n\nþmodule "list/WikiPagesModule" preview%3D%22true%22 þ	2008-08-19 16:25:59
49	\n\nþmodule "login/LoginModule"þ	2008-08-19 16:25:59
47	\n\nþmodule "wiki/pagestagcloud/PagesTagCloudModule" limit%3D%22200%22+target%3D%22system%3Apage-tags%22 þ	2008-08-19 16:25:59
48	\n\n<div style="float:right; width: 50%;">þmodule "wiki/pagestagcloud/PagesTagCloudModule" limit%3D%22200%22+target%3D%22system%3Apage-tags%22 þ</div>\nþmodule "wiki/pagestagcloud/PagesListByTagModule"þ	2008-08-19 16:25:59
50	\n\nþmodule "createaccount2/CreateAccountModule"þ	2008-08-19 16:25:59
51	\n\nþmodule "createaccount2/CreateAccount2Module"þ	2008-08-19 16:25:59
52	\n\nþmodule "createaccount2/CreateAccount3Module"þ	2008-08-19 16:25:59
\.


--
-- Data for Name: page_edit_lock; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.page_edit_lock (lock_id, page_id, mode, section_id, range_start, range_end, page_unix_name, user_id, user_string, session_id, date_started, date_last_accessed, secret, site_id) FROM stdin;
\.


--
-- Data for Name: page_external_link; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.page_external_link (link_id, site_id, page_id, to_url, pinged, ping_status, date) FROM stdin;
\.


--
-- Data for Name: page_inclusion; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.page_inclusion (inclusion_id, including_page_id, included_page_id, included_page_name, site_id) FROM stdin;
\.


--
-- Data for Name: page_link; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) FROM stdin;
1	5	5	\N	2
11	5	6	\N	2
12	11	8	\N	3
14	11	12	\N	3
15	5	13	\N	2
16	5	14	\N	2
18	14	15	\N	2
19	5	15	\N	2
20	5	16	\N	2
21	5	17	\N	2
22	5	18	\N	2
23	5	21	\N	2
24	22	1	\N	1
25	22	22	\N	1
34	22	23	\N	1
35	32	\N	submenu	2
36	32	\N	contact	2
44	36	15	\N	2
45	36	5	\N	2
46	36	32	\N	2
52	22	37	\N	1
53	22	38	\N	1
54	22	3	\N	1
55	22	39	\N	1
56	22	40	\N	1
57	23	1	\N	1
58	23	22	\N	1
59	23	\N	nav:top	1
60	22	41	\N	1
61	22	42	\N	1
62	22	43	\N	1
63	22	44	\N	1
64	44	1	\N	1
65	22	45	\N	1
66	22	46	\N	1
67	22	47	\N	1
\.


--
-- Data for Name: page_metadata; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) FROM stdin;
1	\N	\N	admin:manage	1
2	\N	\N	account:you	1
3	\N	Get a new wiki	new-site	1
4	\N	\N	user:info	1
5	\N	Side	nav:side	1
6	\N	What Is A Wiki Site	what-is-a-wiki-site	1
7	\N	Admin	profile:admin	1
8	\N	\N	admin:manage	1
10	\N	Profile Side	nav:profile-side	1
11	\N	Side	nav:side	1
12	\N	\N	start	1
13	\N	How To Edit Pages - Quickstart	how-to-edit-pages	1
14	\N	Join This Wiki	system:join	1
15	\N	\N	admin:manage	1
16	\N	Page Tags List	system:page-tags-list	1
17	\N	Recent Changes	system:recent-changes	1
18	\N	Members	system:members	1
19	\N	Wiki Search	search:site	1
20	\N	\N	system:page-tags	1
21	\N	List All Pages	system:list-all-pages	1
22	\N	Side	nav:side	1
23	\N	Welcome to Wikidot	start	1
24	\N	Search All Wikis	search:all	1
25	\N	Search	search:site	1
26	\N	Search This Wiki	search:site	1
27	\N	Search Users	search:users	1
28	\N	Forum Categories	forum:start	1
29	\N	Forum Category	forum:category	1
30	\N	Forum Thread	forum:thread	1
31	\N	New Forum Thread	forum:new-thread	1
32	\N	Recent Forum Posts	forum:recent-posts	1
33	\N	Top	nav:top	1
34	\N	Template	profile:template	1
37	\N	Congratulations, welcome to your new wiki!	start	1
38	\N	List of all wikis	system-all:all-sites	1
39	\N	Sites By Tags	system-all:sites-by-tags	1
40	\N	List wikis by tags	system-all:sites-by-tags	1
41	\N	Search	system-all:search	1
42	\N	Activity	system-all:activity	1
43	\N	Activity across all wikis	system-all:activity	1
44	\N	Welcome to your new Wikidot installation!	start	1
45	\N	What Is A Wiki	what-is-a-wiki	1
46	\N	How To Edit Pages	how-to-edit-pages	1
47	\N	Wiki Members	system:members	1
48	\N	How to join this wiki?	system:join	1
49	\N	Recent changes	system:recent-changes	1
50	\N	List all pages	system:list-all-pages	1
51	\N	Page Tags List	system:page-tags-list	1
52	\N	Page Tags	system:page-tags	1
53	\N	Log in	auth:login	1
54	\N	Create account - step 1	auth:newaccount	1
55	\N	Create account - step 2	auth:newaccount2	1
56	\N	Create account - step 3	auth:newaccount3	1
\.


--
-- Data for Name: page_rate_vote; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.page_rate_vote (rate_id, user_id, page_id, rate, date) FROM stdin;
\.


--
-- Data for Name: page_revision; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) FROM stdin;
1	1	1	1	\N	f	f	f	f	f	t	0	f	0	2008-01-24 12:16:34	1	\N		f	1
2	2	2	2	\N	f	f	f	f	f	t	0	f	0	2008-01-24 12:22:02	1	\N		f	1
3	3	3	3	\N	f	f	f	f	f	t	0	f	0	2008-01-24 12:27:10	1	\N		f	1
4	4	4	4	\N	f	f	f	f	f	t	0	f	0	2008-01-24 12:32:21	1	\N		f	1
5	5	5	5	\N	f	f	f	f	f	t	0	f	0	2008-01-25 00:35:20	1	\N		f	2
6	6	6	6	\N	f	f	f	f	f	t	0	f	0	2008-01-25 00:45:30	1	\N		f	2
7	7	7	7	\N	f	f	f	f	f	t	0	f	0	2008-01-25 01:05:59	1	\N		f	3
8	8	8	8	\N	f	f	f	f	f	t	0	f	0	2008-01-25 01:06:39	1	\N		f	3
9	9	9	9	\N	f	f	f	f	f	t	0	f	0	2008-01-25 01:08:10	1	\N		f	1
10	10	10	10	\N	f	f	f	f	f	t	0	f	0	2008-01-25 01:09:41	1	\N		f	3
11	11	11	11	\N	f	f	f	f	f	t	0	f	0	2008-01-25 01:13:41	1	\N		f	3
12	11	12	11	\N	t	f	f	f	f	f	0	f	1	2008-01-25 01:14:31	1	\N		f	3
13	12	13	12	\N	f	f	f	f	f	t	0	f	0	2008-01-25 01:15:35	1	\N		f	3
14	13	14	13	\N	f	f	f	f	f	t	0	f	0	2008-01-29 00:09:59	1	\N		f	2
15	14	15	14	\N	f	f	f	f	f	t	0	f	0	2008-01-29 00:56:59	1	\N		f	2
16	15	16	15	\N	f	f	f	f	f	t	0	f	0	2008-01-29 00:57:39	1	\N		f	2
17	16	17	16	\N	f	f	f	f	f	t	0	f	0	2008-01-29 00:58:44	1	\N		f	2
18	17	18	17	\N	f	f	f	f	f	t	0	f	0	2008-01-29 00:59:14	1	\N		f	2
19	18	19	18	\N	f	f	f	f	f	t	0	f	0	2008-01-29 00:59:40	1	\N		f	2
20	19	20	19	\N	f	f	f	f	f	t	0	f	0	2008-01-29 01:01:49	1	\N		f	2
21	20	21	20	\N	f	f	f	f	f	t	0	f	0	2008-01-29 01:03:43	1	\N		f	2
22	21	22	21	\N	f	f	f	f	f	t	0	f	0	2008-01-29 01:04:52	1	\N		f	2
23	22	23	22	\N	f	f	f	f	f	t	0	f	0	2008-01-29 01:05:47	1	\N		f	1
24	23	24	23	\N	f	f	f	f	f	t	0	f	0	2008-01-29 01:07:41	1	\N		f	1
25	24	25	24	\N	f	f	f	f	f	t	0	f	0	2008-01-29 01:09:17	1	\N		f	1
26	25	26	25	\N	f	f	f	f	f	t	0	f	0	2008-01-29 01:34:40	1	\N		f	1
27	25	26	26	\N	f	t	f	f	f	f	0	f	1	2008-01-29 01:34:57	1	\N		f	1
28	23	27	23	\N	t	f	f	f	f	f	0	f	1	2008-01-29 01:35:41	1	\N		f	1
29	26	28	27	\N	f	f	f	f	f	t	0	f	0	2008-01-29 01:36:56	1	\N		f	1
30	26	29	27	\N	t	f	f	f	f	f	0	f	1	2008-01-29 01:37:12	1	\N		f	1
31	27	30	28	\N	f	f	f	f	f	t	0	f	0	2008-01-29 01:40:23	1	\N		f	2
32	28	31	29	\N	f	f	f	f	f	t	0	f	0	2008-01-29 01:40:59	1	\N		f	2
33	29	32	30	\N	f	f	f	f	f	t	0	f	0	2008-01-29 01:41:32	1	\N		f	2
34	30	33	31	\N	f	f	f	f	f	t	0	f	0	2008-01-29 01:42:10	1	\N		f	2
35	31	34	32	\N	f	f	f	f	f	t	0	f	0	2008-01-29 01:42:42	1	\N		f	2
36	32	35	33	\N	f	f	f	f	f	t	0	f	0	2008-01-29 23:29:51	1	\N		f	2
37	33	36	34	\N	f	f	f	f	f	t	0	f	0	2008-01-29 23:30:18	1	\N		f	2
38	34	37	35	\N	f	f	f	f	f	t	0	f	0	2008-01-30 08:39:24	1	\N		f	2
39	35	38	36	\N	f	f	f	f	f	t	0	f	0	2008-01-30 08:40:31	1	\N		f	2
40	36	39	37	\N	f	f	f	f	f	t	0	f	0	2008-01-30 08:43:22	1	\N		f	2
41	22	40	22	\N	t	f	f	f	f	f	0	f	1	2008-01-30 08:53:14	1	\N		f	1
42	37	41	38	\N	f	f	f	f	f	t	0	f	0	2008-01-30 08:54:56	1	\N		f	1
43	38	42	39	\N	f	f	f	f	f	t	0	f	0	2008-01-30 08:55:33	1	\N		f	1
44	38	43	40	\N	t	t	f	f	f	f	0	f	1	2008-01-30 09:00:00	1	\N		f	1
45	22	44	22	\N	t	f	f	f	f	f	0	f	2	2008-01-30 09:01:50	1	\N		f	1
46	39	45	41	\N	f	f	f	f	f	t	0	f	0	2008-01-30 09:07:05	1	\N		f	1
47	40	46	42	\N	f	f	f	f	f	t	0	f	0	2008-01-30 09:16:38	1	\N		f	1
48	40	47	43	\N	t	t	f	f	f	f	0	f	1	2008-01-30 09:17:40	1	\N		f	1
49	23	48	44	\N	t	t	f	f	f	f	0	f	2	2008-01-30 12:52:23	1	\N		f	1
50	23	49	44	\N	t	f	f	f	f	f	0	f	3	2008-01-30 16:08:02	1	\N		f	1
51	41	50	45	\N	f	f	f	f	f	t	0	f	0	2008-01-30 16:11:56	1	\N		f	1
52	13	51	13	\N	t	f	f	f	f	f	0	f	1	2008-01-30 16:12:40	1	\N		f	2
53	42	52	46	\N	f	f	f	f	f	t	0	f	0	2008-01-30 16:12:48	1	\N		f	1
54	43	53	47	\N	f	f	f	f	f	t	0	f	0	2008-01-30 16:13:32	1	\N		f	1
55	44	54	48	\N	f	f	f	f	f	t	0	f	0	2008-01-30 16:14:13	1	\N		f	1
56	45	55	49	\N	f	f	f	f	f	t	0	f	0	2008-01-30 16:14:41	1	\N		f	1
57	46	56	50	\N	f	f	f	f	f	t	0	f	0	2008-01-30 16:15:22	1	\N		f	1
58	47	57	51	\N	f	f	f	f	f	t	0	f	0	2008-01-30 16:15:56	1	\N		f	1
59	48	58	52	\N	f	f	f	f	f	t	0	f	0	2008-01-30 16:16:22	1	\N		f	1
60	49	59	53	\N	f	f	f	f	f	t	0	f	0	2008-08-19 16:25:58	1	\N	\N	f	1
61	50	60	54	\N	f	f	f	f	f	t	0	f	0	2008-08-19 16:25:58	1	\N	\N	f	1
62	51	61	55	\N	f	f	f	f	f	t	0	f	0	2008-08-19 16:25:58	1	\N	\N	f	1
63	52	62	56	\N	f	f	f	f	f	t	0	f	0	2008-08-19 16:25:58	1	\N	\N	f	1
\.


--
-- Data for Name: page_source; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.page_source (source_id, text) FROM stdin;
1	[[module ManageSite]]
2	[[module Account]]
3	Use this simple form to create a new wiki.\n\nTo admins: you can customize this page by simply clicking "edit" at the bottom of the page.\n\n[[module NewSite]]
4	[[module UserInfo]]
5	* [[[start | Welcome page]]]\n\n* [[[What is a Wiki Site?]]]\n* [[[How to edit pages?]]]\n\n* [[[system: join | How to join this site?]]]\n* [[[system:members | Site members]]] \n\n* [[[system: Recent changes]]]\n* [[[system: List all pages]]]\n* [[[system:page-tags-list|Page Tags]]]\n\n* [[[admin:manage|Site Manager]]]\n\n++ Page tags\n[[module TagCloud minFontSize="80%" maxFontSize="200%"  maxColor="8,8,64" minColor="100,100,128" target="system:page-tags" limit="30"]]\n\n++ Add a new page\n[[module NewPage size="15" button="new page"]]\n\n= [[size 80%]][[[nav:side | edit this panel]]][[/size]]
6	According to [http://en.wikipedia.org/wiki/Wiki Wikipedia], the world largest wiki site:\n\n> A //Wiki// ([ˈwiː.kiː] <wee-kee> or [ˈwɪ.kiː] <wick-ey>) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.\n\nAnd that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.
7	Admin of this Wikidot installation.
8	[[module ManageSite]]
10	The profiles site is used to host user profiles. Each {{profile:username}} page contains a user-editable text that is included in the user's profile page.\n\nIf you are viewing your own profile content page, feel free to edit it. You are the only one allowed to edit this page.
11	* [[[start | Main page]]]\n* [[[admin:manage | Manage this wiki]]]
12	The profiles site is used to host user profiles. Each {{profile:username}} page contains a user-editable text that is included in the user's profile page.\n\n* [[[start | Main page]]]\n* [[[admin:manage | Manage this wiki]]]
13	The purpose of this wiki is to store user profiles.
14	If you are allowed to edit pages in this Site, simply click on //edit// button at the bottom of the page. This will open an editor with a toolbar pallette with options.\n\nTo create a link to a new page, use syntax: {{``[[[new page name]]]``}} or {{``[[[new page name | text to display]]]``}}. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!\n\nAlthough creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit [*http://www.wikidot.com/doc Documentation pages] (at wikidot.com) to learn more.
15	[[note]]\nPlease change this page according to your policy (configure first using [[[admin:manage|Site Manager]]]) and remove this note.\n[[/note]]\n\n+ Who can join?\n\nYou can write here who can become a member of this site.\n\n+ Join!\n\nSo you want to become a member of this site? Tell us why and apply now!\n\n[[module MembershipApply]] \n\nOr, if you already know a "secret password", go for it!\n\n[[module MembershipByPassword]]
16	[[module ManageSite]]
17	[[module TagCloud limit="200" target="system:page-tags"]]\n\n[!--\n\nYou can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module \nBut if you want to keep the tag functionality working - do not remove these modules.\n\n--]
18	[[module SiteChanges]]
19	+ Members:\n\n[[module Members]]\n\n+ Moderators\n\n[[module Members group="moderators"]]\n\n+ Admins\n\n[[module Members group="admins"]]
20	[[module Search]]\n\n[!-- please do not remove or change this page if you want to keep the search function working --]
21	[[div style="float:right; width: 50%;"]]\n[[module TagCloud limit="200" target="system:page-tags"]]\n[[/div]]\n[[module PagesByTag]]\n\n[!--\n\nYou can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module \nBut if you want to keep the tag functionality working - do not remove these modules.\n\n--]
22	[[module Pages preview="true"]]
23	* [[[start | Welcome page]]]\n\n* [[[What is a Wiki Site?]]]\n* [[[How to edit pages?]]]\n\n* [[[system: join | How to join this site?]]]\n* [[[system:members | Site members]]]\n\n* [[[system: Recent changes]]]\n* [[[system: List all pages]]]\n* [[[system:page-tags-list|Page Tags]]]\n\n* [[[admin:manage|Site Manager]]]\n\n++ Page tags\n[[module TagCloud minFontSize="80%" maxFontSize="200%"  maxColor="8,8,64" minColor="100,100,128" target="system:page-tags" limit="30"]]\n\n++ Add a new page\n[[module NewPage size="15" button="new page"]]\n\n= [[size 80%]][[[nav:side | edit this panel]]][[/size]]
24	Welcome to your new Wikidot installation. \n\n+ Search all wikis\n\n[[module SearchAll]]
25	[[module SearchAll]]
26	[[module Search]]
27	Welcome to your new Wikidot installation. \n\n+ Search all wikis\n\n[[module SearchAll]]\n\n+ Search users\n\n[[module SearchUsers]]
28	To look for someone, please enter:\n\n* email address of a person you are looking for (this will look for exact match)\n* any part of the screen name or realname (lists all Users matching the query)\n\n[[module UserSearch]]
29	To look for someone, please enter:\n\n* email address of a person you are looking for (this will look for exact match)\n* any part of the screen name or realname (lists all Users matching the query)\n\n[[module SearchUsers]]
30	[[module ForumStart]]\n[!-- please do not alter this page if you want to keep your forum working --]
31	[[module ForumCategory]]\n\n[!-- please do not alter this page if you want to keep your forum working --]
32	[[module ForumThread]]\n\n[!-- please do not alter this page if you want to keep your forum working --]
33	[[module ForumNewThread]]\n\n[!-- please do not alter this page if you want to keep your forum working --]
34	[[module RecentPosts]]\n\n[!-- please do not alter this page if you want to keep your forum working --]
35	* [# example menu]\n * [[[submenu]]]\n* [[[contact]]]\n\n[!-- top nav menu, use only one bulleted list above --]
36	Profile has not been created (yet).
39	++ If this is your first site\n\nThen there are some things you need to know:\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org].
40	* [[[start | Welcome page]]]\n\n* [[[What is a Wiki?]]]\n* [[[How to edit pages?]]]\n\n+ All wikis\n\n* [[[system-all:activity | Recent activity]]]\n* [[[system-all:all-sites | All wikis]]]\n* [[[system-all:sites-by-tags]]]\n* [[[system-all:search]]]\n\n+ This wiki\n\n* [[[system: join | How to join this site?]]]\n* [[[system:members | Site members]]]\n\n* [[[system: Recent changes]]]\n* [[[system: List all pages]]]\n* [[[system:page-tags-list|Page Tags]]]\n\n* [[[admin:manage|Site Manager]]]\n\n++ Page tags\n[[module TagCloud minFontSize="80%" maxFontSize="200%"  maxColor="8,8,64" minColor="100,100,128" target="system:page-tags" limit="30"]]\n\n++ Add a new page\n[[module NewPage size="15" button="new page"]]\n\n= [[size 80%]][[[nav:side | edit this panel]]][[/size]]
41	Below is the list of public visible Wikis hosted at this service:\n\n[[module ListAllWikis]]
42	[[module SitesTagCloud limit=100]]\n\n\n[[module SitesListByTag]]
43	[[module SitesTagCloud limit="100" target="system-all:sites-by-tags"]]\n\n\n[[module SitesListByTag]]
44	* [[[start | Welcome page]]]\n\n* [[[What is a Wiki?]]]\n* [[[How to edit pages?]]]\n* [[[new-site | Get a new wiki!]]]\n\n+ All wikis\n\n* [[[system-all:activity | Recent activity]]]\n* [[[system-all:all-sites | All wikis]]]\n* [[[system-all:sites-by-tags | Wikis by tags]]]\n* [[[system-all:search | Search]]]\n\n+ This wiki\n\n* [[[system: join | How to join this site?]]]\n* [[[system:members | Site members]]]\n\n* [[[system: Recent changes]]]\n* [[[system: List all pages]]]\n* [[[system:page-tags-list|Page Tags]]]\n\n* [[[admin:manage|Site Manager]]]\n\n++ Page tags\n[[module TagCloud minFontSize="80%" maxFontSize="200%"  maxColor="8,8,64" minColor="100,100,128" target="system:page-tags" limit="30"]]\n\n++ Add a new page\n[[module NewPage size="15" button="new page"]]\n\n= [[size 80%]][[[nav:side | edit this panel]]][[/size]]
45	[[=]]\n+ Search all Wikis\n\nPerform a search through all public and visible wikis.\n\n[[module SearchAll]]\n\n---------------\n\n+ Search users\n\nTo look for someone, please enter:\n\n* email address of a person you are looking for (this will look for exact match)\n* any part of the screen name or realname (lists all Users matching the query)\n\n[[module SearchUsers]]\n\n[[/=]]
46	[[table]]\n[[row]]\n[[cell style="width: 45%; padding-right: 2%; border-right: 1px solid #999;"]]\n\n++ Recent edits (all wikis)\n\n[[module RecentWRevisions]]\n\n[[/cell]]\n[[cell style="width: 45%; padding-left: 2%;"]]\n\n++ Top Sites\n\n[[module MostActiveSites]]\n\n++ Top Forums\n\n[[module MostActiveForums]]\n\n++ New users\n\n[[module NewWUsers]]\n\n++ Some statistics\n\n[[module SomeGlobalStats]]\n\n[[/cell]]\n[[/row]]\n[[/table]]
47	[[table]]\n[[row]]\n[[cell style="width: 45%; padding-right: 2%; border-right: 1px solid #999; vertical-align:top;"]]\n\n++ Recent edits (all wikis)\n\n[[module RecentWRevisions]]\n\n[[/cell]]\n[[cell style="width: 45%; padding-left: 2%; vertical-align:top;"]]\n\n++ Top Sites\n\n[[module MostActiveSites]]\n\n++ Top Forums\n\n[[module MostActiveForums]]\n\n++ New users\n\n[[module NewWUsers]]\n\n++ Some statistics\n\n[[module SomeGlobalStats]]\n\n[[/cell]]\n[[/row]]\n[[/table]]
48	Congratulations, you have successfully installed Wikidot software on your computer!\n\n+ What to do next\n\n++ Customize this wiki\n\nWikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n\n++ Customize default template\n\nDefault initial template for other wikis is located at [[[template-en::]]]. If someone creates a new wiki, this one is cloned to the new address. A good thing to do is to go to [[[template-en::]]] and customize it.\n\n++ Create more templates\n\nSimply create wikis with unix names starting with "template-" (e.g. "template-pl", "template-blog") and your users will be able to choose which wiki they want to start with. \n\n---------------\n\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org].\n\n+ Search all wikis\n\n[[module SearchAll]]\n\n+ Search users\n\n[[module SearchUsers]]
49	Congratulations, you have successfully installed Wikidot software on your computer!\n\n+ What to do next\n\n++ Customize this wiki\n\nWikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n\n++ Customize default template\n\nDefault initial template for other wikis is located at [[[template-en::]]]. If someone creates a new wiki, this one is cloned to the new address. A good thing to do is to go to [[[template-en::]]] and customize it.\n\n++ Create more templates\n\nSimply create wikis with unix names starting with "template-" (e.g. "template-pl", "template-blog") and your users will be able to choose which wiki they want to start with. \n\n++ Visit Wikidot.org\n\nGo to **[http://www.wikidot.org www.wikidot.org]** -- home of the Wikidot software -- for extra documentation, howtos, tips and support.\n\n---------------\n\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org].\n\n+ Search all wikis\n\n[[module SearchAll]]\n\n+ Search users\n\n[[module SearchUsers]]
50	According to [http://en.wikipedia.org/wiki/Wiki Wikipedia], the world largest wiki site:\n\n> A //Wiki// ([ˈwiː.kiː] <wee-kee> or [ˈwɪ.kiː] <wick-ey>) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.\n\nAnd that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.
51	If you are allowed to edit pages in this Site, simply click on //edit// button at the bottom of the page. This will open an editor with a toolbar pallette with options.\n\nTo create a link to a new page, use syntax: {{``[[[new page name]]]``}} or {{``[[[new page name | text to display]]]``}}. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!\n\nAlthough creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit [*http://www.wikidot.org/doc Documentation pages] (at wikidot.org) to learn more.
52	If you are allowed to edit pages in this Site, simply click on //edit// button at the bottom of the page. This will open an editor with a toolbar pallette with options.\n\nTo create a link to a new page, use syntax: {{``[[[new page name]]]``}} or {{``[[[new page name | text to display]]]``}}. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!\n\nAlthough creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit [*http://www.wikidot.org/doc Documentation pages] (at wikidot.org) to learn more.
53	+ Members:\n\n[[module Members]]\n\n+ Moderators\n\n[[module Members group="moderators"]]\n\n+ Admins\n\n[[module Members group="admins"]]
54	[[note]]\nPlease change this page according to your policy (configure first using [[[admin:manage|Site Manager]]]) and remove this note.\n[[/note]]\n\n+ Who can join?\n\nYou can write here who can become a member of this site.\n\n+ Join!\n\nSo you want to become a member of this site? Tell us why and apply now!\n\n[[module MembershipApply]] \n\nOr, if you already know a "secret password", go for it!\n\n[[module MembershipByPassword]]
55	[[module SiteChanges]]
56	[[module Pages preview="true"]]
57	[[module TagCloud limit="200" target="system:page-tags"]]\n\n[!--\n\nYou can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module\nBut if you want to keep the tag functionality working - do not remove these modules.\n\n--]
58	[[div style="float:right; width: 50%;"]]\n[[module TagCloud limit="200" target="system:page-tags"]]\n[[/div]]\n[[module PagesByTag]]\n\n[!--\n\nYou can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module\nBut if you want to keep the tag functionality working - do not remove these modules.\n\n--]
59	[[module LoginModule]]
60	[[module CreateAccount]]
61	[[module CreateAccount2]]
62	[[module CreateAccount3]]
\.


--
-- Data for Name: page_tag; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.page_tag (tag_id, site_id, page_id, tag) FROM stdin;
\.


--
-- Data for Name: petition_campaign; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.petition_campaign (campaign_id, site_id, name, identifier, active, number_signatures, deleted, collect_address, collect_city, collect_state, collect_zip, collect_country, collect_comments, show_city, show_state, show_zip, show_country, show_comments, thank_you_page) FROM stdin;
\.


--
-- Data for Name: petition_signature; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.petition_signature (signature_id, campaign_id, first_name, last_name, address1, address2, zip, city, state, country, country_code, comments, email, confirmed, confirmation_hash, confirmation_url, date) FROM stdin;
\.


--
-- Data for Name: private_message; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.private_message (message_id, from_user_id, to_user_id, subject, body, date, flag, flag_new) FROM stdin;
\.


--
-- Data for Name: private_user_block; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.private_user_block (block_id, user_id, blocked_user_id) FROM stdin;
\.


--
-- Data for Name: profile; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.profile (user_id, real_name, gender, birthday_day, birthday_month, birthday_year, about, location, website, im_aim, im_gadu_gadu, im_google_talk, im_icq, im_jabber, im_msn, im_yahoo, change_screen_name_count) FROM stdin;
1	\N	\N	\N	\N	\N	Wikidot administrator.	\N	\N	\N	\N	\N	\N	\N	\N	\N	0
\.


--
-- Data for Name: simpletodo_list; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.simpletodo_list (list_id, site_id, label, title, data) FROM stdin;
\.


--
-- Data for Name: site; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.site (site_id, name, subtitle, unix_name, description, language, date_created, custom_domain, visible, default_page, private, deleted) FROM stdin;
1	Wikijump	Fighting Ozone Pollution	www	Wikijump host site	en	\N	\N	t	start	f	f
2	Template site (en)	Default template wiki	template-en		en	\N	\N	t	start	f	f
3	User profiles	\N	profiles	\N	en	\N	\N	t	start	f	f
\.


--
-- Data for Name: site_backup; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.site_backup (backup_id, site_id, status, backup_source, backup_files, date, rand) FROM stdin;
\.


--
-- Data for Name: site_settings; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.site_settings (site_id, allow_membership_by_apply, allow_membership_by_password, membership_password, file_storage_size, use_ganalytics, private_landing_page, max_private_members, max_private_viewers, hide_navigation_unauthorized, ssl_mode, openid_enabled, allow_members_invite, max_upload_file_size, enable_all_pingback_out) FROM stdin;
1	t	f	\N	1073741824	f	system:join	50	20	t	\N	f	f	10485760	t
2	f	f		314572800	f	system:join	50	20	t	\N	f	f	10485760	t
3	t	f	\N	314572800	f	system:join	50	20	t	\N	f	f	10485760	t
\.


--
-- Data for Name: site_super_settings; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.site_super_settings (site_id, can_custom_domain) FROM stdin;
1	t
2	t
3	t
\.


--
-- Data for Name: site_tag; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.site_tag (tag_id, site_id, tag) FROM stdin;
1	2	template
\.


--
-- Data for Name: site_viewer; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.site_viewer (viewer_id, site_id, user_id) FROM stdin;
\.


--
-- Data for Name: storage_item; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.storage_item (item_id, date, timeout, data) FROM stdin;
\.


--
-- Data for Name: theme; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) FROM stdin;
1	Base	base	t	\N	\N	f	\N	t	t	0	\N	0
2	Clean	clean	f	1	\N	f	\N	t	t	0	\N	0
4	Flannel	flannel	f	1	\N	f	\N	t	t	0	\N	0
6	Flannel Ocean	flannel-ocean	f	1	\N	f	\N	t	t	0	\N	0
8	Flannel Nature	flannel-nature	f	1	\N	f	\N	t	t	0	\N	0
10	Cappuccino	cappuccino	f	1	\N	f	\N	t	t	0	\N	0
12	Gila	gila	f	1	\N	f	\N	t	t	0	\N	0
14	Co	co	f	1	\N	f	\N	t	t	0	\N	0
15	Flower Blossom	flower-blossom	f	1	\N	f	\N	t	t	0	\N	0
16	Localize	localize	f	1	\N	f	\N	t	t	0	\N	0
20	Webbish	webbish2	f	1	\N	f	\N	t	t	0	\N	0
3	Clean - no side bar	clean-no-side-bar	f	2	2	f	\N	f	t	0	\N	0
5	Flannel - no side bar	flannel-no-side-bar	f	4	4	f	\N	f	t	0	\N	0
7	Flannel Ocean - no side bar	flannel-ocean-no-side-bar	f	6	6	f	\N	f	t	0	\N	0
9	Flannel Nature - no side bar	flannel-nature-no-side-bar	f	8	8	f	\N	f	t	0	\N	0
11	Cappuccino - no side bar	cappuccino-no-side-bar	f	10	10	f	\N	f	t	0	\N	0
13	Gila - no side bar	gila-no-side-bar	f	12	12	f	\N	f	t	0	\N	0
17	Localize - no side bar	localize-no-side-bar	f	16	16	f	\N	f	t	0	\N	0
18	Flower Blossom - no side bar	flower-blossom-no-side-bar	f	15	15	f	\N	f	t	0	\N	0
19	Co - no side bar	co-no-side-bar	f	14	14	f	\N	f	t	0	\N	0
21	Webbish - no side bar	webbish2-no-side-bar	f	20	20	f	\N	f	t	0	\N	0
22	Shiny	shiny	f	1	\N	f	\N	t	t	0	\N	0
23	Shiny - no side bar	shiny-no-side-bar	f	22	22	f	\N	f	t	0	\N	0
24	Bloo	bloo	f	1	\N	f	\N	t	t	0	\N	0
25	Bloo - no side bar	bloo-no-side-bar	f	24	24	f	\N	f	t	0	\N	0
26	Basic	basic	f	1	\N	f	\N	t	t	0	\N	0
28	Black Highlighter	bhl	f	1	\N	t	1	t	t	0		0
27	Sigma-9	sigma9	f	1	\N	t	1	t	t	0		0
\.


--
-- Data for Name: theme_preview; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.theme_preview (theme_id, body) FROM stdin;
\.


--
-- Data for Name: ucookie; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.ucookie (ucookie_id, site_id, session_id, date_granted) FROM stdin;
\.


--
-- Data for Name: unique_string_broker; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.unique_string_broker (last_index) FROM stdin;
127
\.


--
-- Data for Name: user_abuse_flag; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.user_abuse_flag (flag_id, user_id, target_user_id, site_id, site_valid, global_valid) FROM stdin;
\.


--
-- Data for Name: user_block; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.user_block (block_id, site_id, user_id, reason, date_blocked) FROM stdin;
\.


--
-- Data for Name: user_karma; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.user_karma (user_id, points, level) FROM stdin;
1	110	2
\.


--
-- Data for Name: user_settings; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.user_settings (user_id, receive_invitations, receive_pm, notify_online, notify_feed, notify_email, receive_newsletter, receive_digest, allow_site_newsletters_default, max_sites_admin) FROM stdin;
1	t	a    	*	*	\N	t	t	t	3
\.


--
-- Data for Name: watched_forum_thread; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.watched_forum_thread (watched_id, user_id, thread_id) FROM stdin;
\.


--
-- Data for Name: watched_page; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.watched_page (watched_id, user_id, page_id) FROM stdin;
\.


--
-- Name: admin_admin_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.admin_admin_id_seq', 5, true);


--
-- Name: admin_notification_notification_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.admin_notification_notification_id_seq', 3, true);


--
-- Name: anonymous_abuse_flag_flag_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.anonymous_abuse_flag_flag_id_seq', 1, false);


--
-- Name: category_category_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.category_category_id_seq', 59, true);


--
-- Name: category_template_category_template_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.category_template_category_template_id_seq', 1, false);


--
-- Name: comment_comment_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.comment_comment_id_seq', 1, false);


--
-- Name: comment_revision_revision_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.comment_revision_revision_id_seq', 1, false);


--
-- Name: contact_contact_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.contact_contact_id_seq', 1, false);


--
-- Name: domain_redirect_redirect_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.domain_redirect_redirect_id_seq', 1, false);


--
-- Name: email_invitation_invitation_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.email_invitation_invitation_id_seq', 1, false);


--
-- Name: file_file_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.file_file_id_seq', 43, true);


--
-- Name: files_event_file_event_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.files_event_file_event_id_seq', 1, false);


--
-- Name: forum_category_category_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.forum_category_category_id_seq', 2, true);


--
-- Name: forum_group_group_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.forum_group_group_id_seq', 1, true);


--
-- Name: forum_post_post_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.forum_post_post_id_seq', 1, false);


--
-- Name: forum_post_revision_revision_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.forum_post_revision_revision_id_seq', 1, false);


--
-- Name: forum_thread_thread_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.forum_thread_thread_id_seq', 1, true);


--
-- Name: front_forum_feed_feed_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.front_forum_feed_feed_id_seq', 1, false);


--
-- Name: fts_entry_fts_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.fts_entry_fts_id_seq', 175, true);


--
-- Name: global_ip_block_block_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.global_ip_block_block_id_seq', 1, false);


--
-- Name: global_user_block_block_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.global_user_block_block_id_seq', 1, false);


--
-- Name: ip_block_block_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.ip_block_block_id_seq', 1, false);


--
-- Name: license_license_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.license_license_id_seq', 8, true);


--
-- Name: log_event_event_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.log_event_event_id_seq', 348, true);


--
-- Name: member_application_application_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.member_application_application_id_seq', 3, true);


--
-- Name: member_invitation_invitation_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.member_invitation_invitation_id_seq', 1, false);


--
-- Name: member_member_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.member_member_id_seq', 7, true);


--
-- Name: membership_link_link_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.membership_link_link_id_seq', 2, true);


--
-- Name: moderator_moderator_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.moderator_moderator_id_seq', 1, false);


--
-- Name: notification_notification_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.notification_notification_id_seq', 2, true);


--
-- Name: openid_entry_openid_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.openid_entry_openid_id_seq', 1, false);


--
-- Name: ozone_group_group_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.ozone_group_group_id_seq', 1, false);


--
-- Name: ozone_group_permission_modifier_group_permission_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.ozone_group_permission_modifier_group_permission_id_seq', 1, false);


--
-- Name: ozone_permission_permission_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.ozone_permission_permission_id_seq', 1, false);


--
-- Name: ozone_user_group_relation_user_group_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.ozone_user_group_relation_user_group_id_seq', 1, false);


--
-- Name: ozone_user_permission_modifier_user_permission_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.ozone_user_permission_modifier_user_permission_id_seq', 1, false);


--
-- Name: ozone_user_user_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.ozone_user_user_id_seq', 10, true);


--
-- Name: page_abuse_flag_flag_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.page_abuse_flag_flag_id_seq', 1, true);


--
-- Name: page_edit_lock_lock_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.page_edit_lock_lock_id_seq', 246, true);


--
-- Name: page_external_link_link_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.page_external_link_link_id_seq', 150, true);


--
-- Name: page_inclusion_inclusion_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.page_inclusion_inclusion_id_seq', 4, true);


--
-- Name: page_link_link_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.page_link_link_id_seq', 452, true);


--
-- Name: page_metadata_metadata_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.page_metadata_metadata_id_seq', 183, true);


--
-- Name: page_page_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.page_page_id_seq', 173, true);


--
-- Name: page_rate_vote_rate_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.page_rate_vote_rate_id_seq', 1, false);


--
-- Name: page_revision_revision_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.page_revision_revision_id_seq', 340, true);


--
-- Name: page_source_source_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.page_source_source_id_seq', 294, true);


--
-- Name: page_tag_tag_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.page_tag_tag_id_seq', 25, true);


--
-- Name: petition_campaign_campaign_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.petition_campaign_campaign_id_seq', 1, false);


--
-- Name: petition_signature_signature_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.petition_signature_signature_id_seq', 1, false);


--
-- Name: private_message_message_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.private_message_message_id_seq', 1, false);


--
-- Name: private_user_block_block_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.private_user_block_block_id_seq', 1, false);


--
-- Name: simpletodo_list_list_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.simpletodo_list_list_id_seq', 1, false);


--
-- Name: site_backup_backup_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.site_backup_backup_id_seq', 1, true);


--
-- Name: site_site_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.site_site_id_seq', 8, true);


--
-- Name: site_tag_tag_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.site_tag_tag_id_seq', 1, true);


--
-- Name: site_viewer_viewer_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.site_viewer_viewer_id_seq', 1, false);


--
-- Name: theme_theme_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.theme_theme_id_seq', 28, true);


--
-- Name: user_abuse_flag_flag_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.user_abuse_flag_flag_id_seq', 1, false);


--
-- Name: user_block_block_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.user_block_block_id_seq', 1, false);


--
-- Name: watched_forum_thread_watched_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.watched_forum_thread_watched_id_seq', 1, false);


--
-- Name: watched_page_watched_id_seq; Type: SEQUENCE SET; Schema: public; Owner: wikijump
--

SELECT pg_catalog.setval('public.watched_page_watched_id_seq', 1, false);


--
-- Name: admin admin__site_id__user_id__unique; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.admin
    ADD CONSTRAINT admin__site_id__user_id__unique UNIQUE (site_id, user_id);


--
-- Name: admin_notification admin_notification_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.admin_notification
    ADD CONSTRAINT admin_notification_pkey PRIMARY KEY (notification_id);


--
-- Name: admin admin_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.admin
    ADD CONSTRAINT admin_pkey PRIMARY KEY (admin_id);


--
-- Name: anonymous_abuse_flag anonymous_abuse_flag_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.anonymous_abuse_flag
    ADD CONSTRAINT anonymous_abuse_flag_pkey PRIMARY KEY (flag_id);


--
-- Name: category category_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.category
    ADD CONSTRAINT category_pkey PRIMARY KEY (category_id);


--
-- Name: category_template category_template_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.category_template
    ADD CONSTRAINT category_template_pkey PRIMARY KEY (category_template_id);


--
-- Name: comment comment_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.comment
    ADD CONSTRAINT comment_pkey PRIMARY KEY (comment_id);


--
-- Name: comment_revision comment_revision_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.comment_revision
    ADD CONSTRAINT comment_revision_pkey PRIMARY KEY (revision_id);


--
-- Name: contact contact__unique; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.contact
    ADD CONSTRAINT contact__unique UNIQUE (user_id, target_user_id);


--
-- Name: contact contact_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.contact
    ADD CONSTRAINT contact_pkey PRIMARY KEY (contact_id);


--
-- Name: domain_redirect domain_redirect__unique; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.domain_redirect
    ADD CONSTRAINT domain_redirect__unique UNIQUE (site_id, url);


--
-- Name: domain_redirect domain_redirect_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.domain_redirect
    ADD CONSTRAINT domain_redirect_pkey PRIMARY KEY (redirect_id);


--
-- Name: email_invitation email_invitation_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.email_invitation
    ADD CONSTRAINT email_invitation_pkey PRIMARY KEY (invitation_id);


--
-- Name: file file_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.file
    ADD CONSTRAINT file_pkey PRIMARY KEY (file_id);


--
-- Name: files_event files_event_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.files_event
    ADD CONSTRAINT files_event_pkey PRIMARY KEY (file_event_id);


--
-- Name: form_submission_key form_submission_key_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.form_submission_key
    ADD CONSTRAINT form_submission_key_pkey PRIMARY KEY (key_id);


--
-- Name: forum_category forum_category_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_category
    ADD CONSTRAINT forum_category_pkey PRIMARY KEY (category_id);


--
-- Name: forum_group forum_group_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_group
    ADD CONSTRAINT forum_group_pkey PRIMARY KEY (group_id);


--
-- Name: forum_post forum_post_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_post
    ADD CONSTRAINT forum_post_pkey PRIMARY KEY (post_id);


--
-- Name: forum_post_revision forum_post_revision_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_post_revision
    ADD CONSTRAINT forum_post_revision_pkey PRIMARY KEY (revision_id);


--
-- Name: forum_settings forum_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_settings
    ADD CONSTRAINT forum_settings_pkey PRIMARY KEY (site_id);


--
-- Name: forum_thread forum_thread_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_thread
    ADD CONSTRAINT forum_thread_pkey PRIMARY KEY (thread_id);


--
-- Name: front_forum_feed front_forum_feed_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.front_forum_feed
    ADD CONSTRAINT front_forum_feed_pkey PRIMARY KEY (feed_id);


--
-- Name: fts_entry fts_entry_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.fts_entry
    ADD CONSTRAINT fts_entry_pkey PRIMARY KEY (fts_id);


--
-- Name: global_ip_block global_ip_block_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.global_ip_block
    ADD CONSTRAINT global_ip_block_pkey PRIMARY KEY (block_id);


--
-- Name: global_user_block global_user_block_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.global_user_block
    ADD CONSTRAINT global_user_block_pkey PRIMARY KEY (block_id);


--
-- Name: ip_block ip_block_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ip_block
    ADD CONSTRAINT ip_block_pkey PRIMARY KEY (block_id);


--
-- Name: license license_name_key; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.license
    ADD CONSTRAINT license_name_key UNIQUE (name);


--
-- Name: license license_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.license
    ADD CONSTRAINT license_pkey PRIMARY KEY (license_id);


--
-- Name: log_event log_event_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.log_event
    ADD CONSTRAINT log_event_pkey PRIMARY KEY (event_id);


--
-- Name: member member__unique; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.member
    ADD CONSTRAINT member__unique UNIQUE (site_id, user_id);


--
-- Name: member_application member_application__unique; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.member_application
    ADD CONSTRAINT member_application__unique UNIQUE (site_id, user_id);


--
-- Name: member_application member_application_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.member_application
    ADD CONSTRAINT member_application_pkey PRIMARY KEY (application_id);


--
-- Name: member_invitation member_invitation_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.member_invitation
    ADD CONSTRAINT member_invitation_pkey PRIMARY KEY (invitation_id);


--
-- Name: member member_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.member
    ADD CONSTRAINT member_pkey PRIMARY KEY (member_id);


--
-- Name: membership_link membership_link_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.membership_link
    ADD CONSTRAINT membership_link_pkey PRIMARY KEY (link_id);


--
-- Name: moderator moderator__unique; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.moderator
    ADD CONSTRAINT moderator__unique UNIQUE (site_id, user_id);


--
-- Name: moderator moderator_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.moderator
    ADD CONSTRAINT moderator_pkey PRIMARY KEY (moderator_id);


--
-- Name: notification notification_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.notification
    ADD CONSTRAINT notification_pkey PRIMARY KEY (notification_id);


--
-- Name: openid_entry openid_entry_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.openid_entry
    ADD CONSTRAINT openid_entry_pkey PRIMARY KEY (openid_id);


--
-- Name: ozone_group ozone_group_name_key; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_group
    ADD CONSTRAINT ozone_group_name_key UNIQUE (name);


--
-- Name: ozone_group_permission_modifier ozone_group_permission_modifier_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_group_permission_modifier
    ADD CONSTRAINT ozone_group_permission_modifier_pkey PRIMARY KEY (group_permission_id);


--
-- Name: ozone_group ozone_group_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_group
    ADD CONSTRAINT ozone_group_pkey PRIMARY KEY (group_id);


--
-- Name: ozone_lock ozone_lock_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_lock
    ADD CONSTRAINT ozone_lock_pkey PRIMARY KEY (key);


--
-- Name: ozone_permission ozone_permission_name_key; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_permission
    ADD CONSTRAINT ozone_permission_name_key UNIQUE (name);


--
-- Name: ozone_permission ozone_permission_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_permission
    ADD CONSTRAINT ozone_permission_pkey PRIMARY KEY (permission_id);


--
-- Name: ozone_session ozone_session_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_session
    ADD CONSTRAINT ozone_session_pkey PRIMARY KEY (session_id);


--
-- Name: ozone_user_group_relation ozone_user_group_relation_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_user_group_relation
    ADD CONSTRAINT ozone_user_group_relation_pkey PRIMARY KEY (user_group_id);


--
-- Name: ozone_user ozone_user_name_key; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_user
    ADD CONSTRAINT ozone_user_name_key UNIQUE (name);


--
-- Name: ozone_user_permission_modifier ozone_user_permission_modifier_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_user_permission_modifier
    ADD CONSTRAINT ozone_user_permission_modifier_pkey PRIMARY KEY (user_permission_id);


--
-- Name: ozone_user ozone_user_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_user
    ADD CONSTRAINT ozone_user_pkey PRIMARY KEY (user_id);


--
-- Name: ozone_user ozone_user_unix_name_key; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_user
    ADD CONSTRAINT ozone_user_unix_name_key UNIQUE (unix_name);


--
-- Name: page page__unique; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page
    ADD CONSTRAINT page__unique UNIQUE (site_id, unix_name);


--
-- Name: page_abuse_flag page_abuse_flag_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_abuse_flag
    ADD CONSTRAINT page_abuse_flag_pkey PRIMARY KEY (flag_id);


--
-- Name: page_compiled page_compiled_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_compiled
    ADD CONSTRAINT page_compiled_pkey PRIMARY KEY (page_id);


--
-- Name: page_edit_lock page_edit_lock_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_edit_lock
    ADD CONSTRAINT page_edit_lock_pkey PRIMARY KEY (lock_id);


--
-- Name: page_external_link page_external_link_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_external_link
    ADD CONSTRAINT page_external_link_pkey PRIMARY KEY (link_id);


--
-- Name: page_inclusion page_inclusion__unique; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_inclusion
    ADD CONSTRAINT page_inclusion__unique UNIQUE (including_page_id, included_page_id, included_page_name);


--
-- Name: page_inclusion page_inclusion_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_inclusion
    ADD CONSTRAINT page_inclusion_pkey PRIMARY KEY (inclusion_id);


--
-- Name: page_link page_link__unique; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_link
    ADD CONSTRAINT page_link__unique UNIQUE (from_page_id, to_page_id, to_page_name);


--
-- Name: page_link page_link_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_link
    ADD CONSTRAINT page_link_pkey PRIMARY KEY (link_id);


--
-- Name: page_metadata page_metadata_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_metadata
    ADD CONSTRAINT page_metadata_pkey PRIMARY KEY (metadata_id);


--
-- Name: page page_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page
    ADD CONSTRAINT page_pkey PRIMARY KEY (page_id);


--
-- Name: page_rate_vote page_rate_vote_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_rate_vote
    ADD CONSTRAINT page_rate_vote_pkey PRIMARY KEY (rate_id);


--
-- Name: page_rate_vote page_rate_vote_user_id_key; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_rate_vote
    ADD CONSTRAINT page_rate_vote_user_id_key UNIQUE (user_id, page_id);


--
-- Name: page_revision page_revision_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_revision
    ADD CONSTRAINT page_revision_pkey PRIMARY KEY (revision_id);


--
-- Name: page_source page_source_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_source
    ADD CONSTRAINT page_source_pkey PRIMARY KEY (source_id);


--
-- Name: page_tag page_tag_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_tag
    ADD CONSTRAINT page_tag_pkey PRIMARY KEY (tag_id);


--
-- Name: petition_campaign petition_campaign_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.petition_campaign
    ADD CONSTRAINT petition_campaign_pkey PRIMARY KEY (campaign_id);


--
-- Name: petition_signature petition_signature_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.petition_signature
    ADD CONSTRAINT petition_signature_pkey PRIMARY KEY (signature_id);


--
-- Name: private_message private_message_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.private_message
    ADD CONSTRAINT private_message_pkey PRIMARY KEY (message_id);


--
-- Name: private_user_block private_user_block__unique; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.private_user_block
    ADD CONSTRAINT private_user_block__unique UNIQUE (user_id, blocked_user_id);


--
-- Name: private_user_block private_user_block_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.private_user_block
    ADD CONSTRAINT private_user_block_pkey PRIMARY KEY (block_id);


--
-- Name: profile profile_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.profile
    ADD CONSTRAINT profile_pkey PRIMARY KEY (user_id);


--
-- Name: simpletodo_list simpletodo_list__unique; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.simpletodo_list
    ADD CONSTRAINT simpletodo_list__unique UNIQUE (site_id, label);


--
-- Name: simpletodo_list simpletodo_list_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.simpletodo_list
    ADD CONSTRAINT simpletodo_list_pkey PRIMARY KEY (list_id);


--
-- Name: site_backup site_backup_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site_backup
    ADD CONSTRAINT site_backup_pkey PRIMARY KEY (backup_id);


--
-- Name: site site_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site
    ADD CONSTRAINT site_pkey PRIMARY KEY (site_id);


--
-- Name: site_settings site_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site_settings
    ADD CONSTRAINT site_settings_pkey PRIMARY KEY (site_id);


--
-- Name: site_super_settings site_super_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site_super_settings
    ADD CONSTRAINT site_super_settings_pkey PRIMARY KEY (site_id);


--
-- Name: site_tag site_tag__unique; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site_tag
    ADD CONSTRAINT site_tag__unique UNIQUE (site_id, tag);


--
-- Name: site_tag site_tag_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site_tag
    ADD CONSTRAINT site_tag_pkey PRIMARY KEY (tag_id);


--
-- Name: site_viewer site_viewer_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site_viewer
    ADD CONSTRAINT site_viewer_pkey PRIMARY KEY (viewer_id);


--
-- Name: storage_item storage_item_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.storage_item
    ADD CONSTRAINT storage_item_pkey PRIMARY KEY (item_id);


--
-- Name: theme theme_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.theme
    ADD CONSTRAINT theme_pkey PRIMARY KEY (theme_id);


--
-- Name: theme_preview theme_preview_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.theme_preview
    ADD CONSTRAINT theme_preview_pkey PRIMARY KEY (theme_id);


--
-- Name: ucookie ucookie_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ucookie
    ADD CONSTRAINT ucookie_pkey PRIMARY KEY (ucookie_id);


--
-- Name: user_abuse_flag user_abuse_flag_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.user_abuse_flag
    ADD CONSTRAINT user_abuse_flag_pkey PRIMARY KEY (flag_id);


--
-- Name: user_block user_block__unique; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.user_block
    ADD CONSTRAINT user_block__unique UNIQUE (site_id, user_id);


--
-- Name: user_block user_block_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.user_block
    ADD CONSTRAINT user_block_pkey PRIMARY KEY (block_id);


--
-- Name: user_karma user_karma_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.user_karma
    ADD CONSTRAINT user_karma_pkey PRIMARY KEY (user_id);


--
-- Name: user_settings user_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.user_settings
    ADD CONSTRAINT user_settings_pkey PRIMARY KEY (user_id);


--
-- Name: watched_page wached_page__unique; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.watched_page
    ADD CONSTRAINT wached_page__unique UNIQUE (user_id, page_id);


--
-- Name: watched_forum_thread watched_forum_thread__unique; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.watched_forum_thread
    ADD CONSTRAINT watched_forum_thread__unique UNIQUE (user_id, thread_id);


--
-- Name: watched_forum_thread watched_forum_thread_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.watched_forum_thread
    ADD CONSTRAINT watched_forum_thread_pkey PRIMARY KEY (watched_id);


--
-- Name: watched_page watched_page_pkey; Type: CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.watched_page
    ADD CONSTRAINT watched_page_pkey PRIMARY KEY (watched_id);


--
-- Name: admin_notification__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX admin_notification__site_id__idx ON public.admin_notification USING btree (site_id);


--
-- Name: anonymous_abuse_flag__address__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX anonymous_abuse_flag__address__idx ON public.anonymous_abuse_flag USING btree (address);


--
-- Name: anonymous_abuse_flag__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX anonymous_abuse_flag__site_id__idx ON public.anonymous_abuse_flag USING btree (site_id);


--
-- Name: category__name__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX category__name__idx ON public.category USING btree (name);


--
-- Name: category__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX category__site_id__idx ON public.category USING btree (site_id);


--
-- Name: email_invitation__site_id; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX email_invitation__site_id ON public.email_invitation USING btree (site_id);


--
-- Name: email_invitation__user_id; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX email_invitation__user_id ON public.email_invitation USING btree (user_id);


--
-- Name: file__page_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX file__page_id__idx ON public.file USING btree (page_id);


--
-- Name: file__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX file__site_id__idx ON public.file USING btree (site_id);


--
-- Name: fki_forum_category__forum_post; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX fki_forum_category__forum_post ON public.forum_category USING btree (last_post_id);


--
-- Name: forum_category__group_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX forum_category__group_id__idx ON public.forum_category USING btree (group_id);


--
-- Name: forum_category__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX forum_category__site_id__idx ON public.forum_category USING btree (site_id);


--
-- Name: forum_group__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX forum_group__site_id__idx ON public.forum_group USING btree (site_id);


--
-- Name: forum_post__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX forum_post__site_id__idx ON public.forum_post USING btree (site_id);


--
-- Name: forum_post__thread_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX forum_post__thread_id__idx ON public.forum_post USING btree (thread_id);


--
-- Name: forum_post__user_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX forum_post__user_id__idx ON public.forum_post USING btree (user_id);


--
-- Name: forum_post_revision__post_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX forum_post_revision__post_id__idx ON public.forum_post_revision USING btree (post_id);


--
-- Name: forum_thread__category_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX forum_thread__category_id__idx ON public.forum_thread USING btree (category_id);


--
-- Name: forum_thread__last_post_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX forum_thread__last_post_id__idx ON public.forum_thread USING btree (last_post_id);


--
-- Name: forum_thread__page_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX forum_thread__page_id__idx ON public.forum_thread USING btree (page_id);


--
-- Name: forum_thread__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX forum_thread__site_id__idx ON public.forum_thread USING btree (site_id);


--
-- Name: forum_thread__user_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX forum_thread__user_id__idx ON public.forum_thread USING btree (user_id);


--
-- Name: front_forum_feed__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX front_forum_feed__site_id__idx ON public.front_forum_feed USING btree (site_id);


--
-- Name: fts_entry__forum_thread__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX fts_entry__forum_thread__idx ON public.fts_entry USING btree (thread_id);


--
-- Name: fts_entry__page_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX fts_entry__page_id__idx ON public.fts_entry USING btree (page_id);


--
-- Name: fts_entry__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX fts_entry__site_id__idx ON public.fts_entry USING btree (site_id);


--
-- Name: fts_entry__vector__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX fts_entry__vector__idx ON public.fts_entry USING gist (vector);


--
-- Name: ip_block__ip__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX ip_block__ip__idx ON public.ip_block USING btree (ip);


--
-- Name: ip_block__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX ip_block__site_id__idx ON public.ip_block USING btree (site_id);


--
-- Name: log_event__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX log_event__site_id__idx ON public.log_event USING btree (site_id);


--
-- Name: log_event__type__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX log_event__type__idx ON public.log_event USING btree (type);


--
-- Name: member__site_id_user_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE UNIQUE INDEX member__site_id_user_id__idx ON public.member USING btree (site_id, user_id);


--
-- Name: member_application__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX member_application__site_id__idx ON public.member_application USING btree (site_id);


--
-- Name: member_application__user_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX member_application__user_id__idx ON public.member_application USING btree (user_id);


--
-- Name: member_invitation__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX member_invitation__site_id__idx ON public.member_invitation USING btree (site_id);


--
-- Name: member_invitation__user_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX member_invitation__user_id__idx ON public.member_invitation USING btree (user_id);


--
-- Name: moderator__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX moderator__site_id__idx ON public.moderator USING btree (site_id);


--
-- Name: moderator__user_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX moderator__user_id__idx ON public.moderator USING btree (user_id);


--
-- Name: notification__user_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX notification__user_id__idx ON public.notification USING btree (user_id);


--
-- Name: ozone_session__user_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX ozone_session__user_id__idx ON public.ozone_session USING btree (user_id);


--
-- Name: ozone_user__name__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE UNIQUE INDEX ozone_user__name__idx ON public.ozone_user USING btree (name);


--
-- Name: ozone_user__nick_name__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE UNIQUE INDEX ozone_user__nick_name__idx ON public.ozone_user USING btree (nick_name);


--
-- Name: ozone_user__unix_name__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE UNIQUE INDEX ozone_user__unix_name__idx ON public.ozone_user USING btree (unix_name);


--
-- Name: page__category_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX page__category_id__idx ON public.page USING btree (category_id);


--
-- Name: page__parent_page_id; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX page__parent_page_id ON public.page USING btree (parent_page_id);


--
-- Name: page__revision_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX page__revision_id__idx ON public.page USING btree (revision_id);


--
-- Name: page__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX page__site_id__idx ON public.page USING btree (site_id);


--
-- Name: page__unix_name__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX page__unix_name__idx ON public.page USING btree (unix_name);


--
-- Name: page_abuse_flag__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX page_abuse_flag__site_id__idx ON public.page_abuse_flag USING btree (site_id);


--
-- Name: page_edit_lock__page_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX page_edit_lock__page_id__idx ON public.page_edit_lock USING btree (page_id);


--
-- Name: page_edit_lock__site_id_page_unix_name; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX page_edit_lock__site_id_page_unix_name ON public.page_edit_lock USING btree (site_id, page_unix_name);


--
-- Name: page_edit_lock__user_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX page_edit_lock__user_id__idx ON public.page_edit_lock USING btree (user_id);


--
-- Name: page_inclusion__site_id; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX page_inclusion__site_id ON public.page_inclusion USING btree (site_id);


--
-- Name: page_link__site_id; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX page_link__site_id ON public.page_link USING btree (site_id);


--
-- Name: page_revision__page_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX page_revision__page_id__idx ON public.page_revision USING btree (page_id);


--
-- Name: page_revision__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX page_revision__site_id__idx ON public.page_revision USING btree (site_id);


--
-- Name: page_revision__user_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX page_revision__user_id__idx ON public.page_revision USING btree (user_id);


--
-- Name: page_tag__page_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX page_tag__page_id__idx ON public.page_tag USING btree (page_id);


--
-- Name: page_tag__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX page_tag__site_id__idx ON public.page_tag USING btree (site_id);


--
-- Name: private_message__from_user_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX private_message__from_user_id__idx ON public.private_message USING btree (from_user_id);


--
-- Name: private_message__to_user_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX private_message__to_user_id__idx ON public.private_message USING btree (to_user_id);


--
-- Name: ront_forum_feed__page_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX ront_forum_feed__page_id__idx ON public.front_forum_feed USING btree (page_id);


--
-- Name: simpletodo_list__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX simpletodo_list__site_id__idx ON public.simpletodo_list USING btree (site_id);


--
-- Name: site__custom_domain__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX site__custom_domain__idx ON public.site USING btree (custom_domain);


--
-- Name: site__unix_name__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE UNIQUE INDEX site__unix_name__idx ON public.site USING btree (unix_name);


--
-- Name: site__visible__private__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX site__visible__private__idx ON public.site USING btree (visible, private);


--
-- Name: ucookie__session_id_idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX ucookie__session_id_idx ON public.ucookie USING btree (session_id);


--
-- Name: ucookie__site_id; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX ucookie__site_id ON public.ucookie USING btree (site_id);


--
-- Name: user_abuse_flag__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX user_abuse_flag__site_id__idx ON public.user_abuse_flag USING btree (site_id);


--
-- Name: user_block__site_id__idx; Type: INDEX; Schema: public; Owner: wikijump
--

CREATE INDEX user_block__site_id__idx ON public.user_block USING btree (site_id);


--
-- Name: ozone_user get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.ozone_user DO  SELECT currval('public.ozone_user_user_id_seq'::regclass) AS id;


--
-- Name: ozone_group get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.ozone_group DO  SELECT currval('public.ozone_group_group_id_seq'::regclass) AS id;


--
-- Name: ozone_permission get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.ozone_permission DO  SELECT currval('public.ozone_permission_permission_id_seq'::regclass) AS id;


--
-- Name: ozone_user_group_relation get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.ozone_user_group_relation DO  SELECT currval('public.ozone_user_group_relation_user_group_id_seq'::regclass) AS id;


--
-- Name: ozone_user_permission_modifier get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.ozone_user_permission_modifier DO  SELECT currval('public.ozone_user_permission_modifier_user_permission_id_seq'::regclass) AS id;


--
-- Name: ozone_group_permission_modifier get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.ozone_group_permission_modifier DO  SELECT currval('public.ozone_group_permission_modifier_group_permission_id_seq'::regclass) AS id;


--
-- Name: site get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.site DO  SELECT currval('public.site_site_id_seq'::regclass) AS id;


--
-- Name: site_tag get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.site_tag DO  SELECT currval('public.site_tag_tag_id_seq'::regclass) AS id;


--
-- Name: category get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.category DO  SELECT currval('public.category_category_id_seq'::regclass) AS id;


--
-- Name: page get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.page DO  SELECT currval('public.page_page_id_seq'::regclass) AS id;


--
-- Name: page_revision get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.page_revision DO  SELECT currval('public.page_revision_revision_id_seq'::regclass) AS id;


--
-- Name: page_source get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.page_source DO  SELECT currval('public.page_source_source_id_seq'::regclass) AS id;


--
-- Name: page_metadata get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.page_metadata DO  SELECT currval('public.page_metadata_metadata_id_seq'::regclass) AS id;


--
-- Name: fts_entry get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.fts_entry DO  SELECT currval('public.fts_entry_fts_id_seq'::regclass) AS id;


--
-- Name: file get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.file DO  SELECT currval('public.file_file_id_seq'::regclass) AS id;


--
-- Name: files_event get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.files_event DO  SELECT currval('public.files_event_file_event_id_seq'::regclass) AS id;


--
-- Name: page_link get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.page_link DO  SELECT currval('public.page_link_link_id_seq'::regclass) AS id;


--
-- Name: page_inclusion get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.page_inclusion DO  SELECT currval('public.page_inclusion_inclusion_id_seq'::regclass) AS id;


--
-- Name: member get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.member DO  SELECT currval('public.member_member_id_seq'::regclass) AS id;


--
-- Name: admin get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.admin DO  SELECT currval('public.admin_admin_id_seq'::regclass) AS id;


--
-- Name: moderator get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.moderator DO  SELECT currval('public.moderator_moderator_id_seq'::regclass) AS id;


--
-- Name: member_application get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.member_application DO  SELECT currval('public.member_application_application_id_seq'::regclass) AS id;


--
-- Name: member_invitation get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.member_invitation DO  SELECT currval('public.member_invitation_invitation_id_seq'::regclass) AS id;


--
-- Name: page_edit_lock get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.page_edit_lock DO  SELECT currval('public.page_edit_lock_lock_id_seq'::regclass) AS id;


--
-- Name: theme get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.theme DO  SELECT currval('public.theme_theme_id_seq'::regclass) AS id;


--
-- Name: license get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.license DO  SELECT currval('public.license_license_id_seq'::regclass) AS id;


--
-- Name: notification get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.notification DO  SELECT currval('public.notification_notification_id_seq'::regclass) AS id;


--
-- Name: private_message get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.private_message DO  SELECT currval('public.private_message_message_id_seq'::regclass) AS id;


--
-- Name: global_ip_block get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.global_ip_block DO  SELECT currval('public.global_ip_block_block_id_seq'::regclass) AS id;


--
-- Name: ip_block get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.ip_block DO  SELECT currval('public.ip_block_block_id_seq'::regclass) AS id;


--
-- Name: global_user_block get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.global_user_block DO  SELECT currval('public.global_user_block_block_id_seq'::regclass) AS id;


--
-- Name: user_block get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.user_block DO  SELECT currval('public.user_block_block_id_seq'::regclass) AS id;


--
-- Name: private_user_block get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.private_user_block DO  SELECT currval('public.private_user_block_block_id_seq'::regclass) AS id;


--
-- Name: watched_page get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.watched_page DO  SELECT currval('public.watched_page_watched_id_seq'::regclass) AS id;


--
-- Name: watched_forum_thread get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.watched_forum_thread DO  SELECT currval('public.watched_forum_thread_watched_id_seq'::regclass) AS id;


--
-- Name: page_abuse_flag get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.page_abuse_flag DO  SELECT currval('public.page_abuse_flag_flag_id_seq'::regclass) AS id;


--
-- Name: user_abuse_flag get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.user_abuse_flag DO  SELECT currval('public.user_abuse_flag_flag_id_seq'::regclass) AS id;


--
-- Name: anonymous_abuse_flag get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.anonymous_abuse_flag DO  SELECT currval('public.anonymous_abuse_flag_flag_id_seq'::regclass) AS id;


--
-- Name: admin_notification get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.admin_notification DO  SELECT currval('public.admin_notification_notification_id_seq'::regclass) AS id;


--
-- Name: forum_group get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.forum_group DO  SELECT currval('public.forum_group_group_id_seq'::regclass) AS id;


--
-- Name: forum_category get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.forum_category DO  SELECT currval('public.forum_category_category_id_seq'::regclass) AS id;


--
-- Name: forum_thread get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.forum_thread DO  SELECT currval('public.forum_thread_thread_id_seq'::regclass) AS id;


--
-- Name: forum_post get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.forum_post DO  SELECT currval('public.forum_post_post_id_seq'::regclass) AS id;


--
-- Name: forum_post_revision get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.forum_post_revision DO  SELECT currval('public.forum_post_revision_revision_id_seq'::regclass) AS id;


--
-- Name: front_forum_feed get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.front_forum_feed DO  SELECT currval('public.front_forum_feed_feed_id_seq'::regclass) AS id;


--
-- Name: contact get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.contact DO  SELECT currval('public.contact_contact_id_seq'::regclass) AS id;


--
-- Name: page_rate_vote get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.page_rate_vote DO  SELECT currval('public.page_rate_vote_rate_id_seq'::regclass) AS id;


--
-- Name: email_invitation get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.email_invitation DO  SELECT currval('public.email_invitation_invitation_id_seq'::regclass) AS id;


--
-- Name: site_backup get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.site_backup DO  SELECT currval('public.site_backup_backup_id_seq'::regclass) AS id;


--
-- Name: domain_redirect get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.domain_redirect DO  SELECT currval('public.domain_redirect_redirect_id_seq'::regclass) AS id;


--
-- Name: site_viewer get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.site_viewer DO  SELECT currval('public.site_viewer_viewer_id_seq'::regclass) AS id;


--
-- Name: openid_entry get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.openid_entry DO  SELECT currval('public.openid_entry_openid_id_seq'::regclass) AS id;


--
-- Name: membership_link get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.membership_link DO  SELECT currval('public.membership_link_link_id_seq'::regclass) AS id;


--
-- Name: petition_campaign get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.petition_campaign DO  SELECT currval('public.petition_campaign_campaign_id_seq'::regclass) AS id;


--
-- Name: petition_signature get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.petition_signature DO  SELECT currval('public.petition_signature_signature_id_seq'::regclass) AS id;


--
-- Name: simpletodo_list get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.simpletodo_list DO  SELECT currval('public.simpletodo_list_list_id_seq'::regclass) AS id;


--
-- Name: comment get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.comment DO  SELECT currval('public.comment_comment_id_seq'::regclass) AS id;


--
-- Name: comment_revision get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.comment_revision DO  SELECT currval('public.comment_revision_revision_id_seq'::regclass) AS id;


--
-- Name: page_external_link get_pkey_on_insert; Type: RULE; Schema: public; Owner: wikijump
--

CREATE RULE get_pkey_on_insert AS
    ON INSERT TO public.page_external_link DO  SELECT currval('public.page_external_link_link_id_seq'::regclass) AS id;


--
-- Name: admin admin__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.admin
    ADD CONSTRAINT admin__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: admin admin__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.admin
    ADD CONSTRAINT admin__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: admin_notification admin_notification__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.admin_notification
    ADD CONSTRAINT admin_notification__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: anonymous_abuse_flag anonymous_abuse_flag__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.anonymous_abuse_flag
    ADD CONSTRAINT anonymous_abuse_flag__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: anonymous_abuse_flag anonymous_abuse_flag__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.anonymous_abuse_flag
    ADD CONSTRAINT anonymous_abuse_flag__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: category category__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.category
    ADD CONSTRAINT category__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: contact contact__ozone_user__tagret_user_id; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.contact
    ADD CONSTRAINT contact__ozone_user__tagret_user_id FOREIGN KEY (target_user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: contact contact__ozone_user__user_id; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.contact
    ADD CONSTRAINT contact__ozone_user__user_id FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: domain_redirect domain_redirect__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.domain_redirect
    ADD CONSTRAINT domain_redirect__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: email_invitation email_inviation__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.email_invitation
    ADD CONSTRAINT email_inviation__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id);


--
-- Name: email_invitation email_invitation__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.email_invitation
    ADD CONSTRAINT email_invitation__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: file file__page; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.file
    ADD CONSTRAINT file__page FOREIGN KEY (page_id) REFERENCES public.page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: file file__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.file
    ADD CONSTRAINT file__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: file file__user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.file
    ADD CONSTRAINT file__user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: forum_category forum_category__forum_group; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_category
    ADD CONSTRAINT forum_category__forum_group FOREIGN KEY (group_id) REFERENCES public.forum_group(group_id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- Name: forum_category forum_category__forum_post; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_category
    ADD CONSTRAINT forum_category__forum_post FOREIGN KEY (last_post_id) REFERENCES public.forum_post(post_id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: forum_category forum_category__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_category
    ADD CONSTRAINT forum_category__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: forum_group forum_group__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_group
    ADD CONSTRAINT forum_group__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: forum_post forum_post__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_post
    ADD CONSTRAINT forum_post__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id);


--
-- Name: forum_post forum_post__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_post
    ADD CONSTRAINT forum_post__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: forum_settings forum_settings__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_settings
    ADD CONSTRAINT forum_settings__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: forum_thread forum_thread__forum_category; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_thread
    ADD CONSTRAINT forum_thread__forum_category FOREIGN KEY (category_id) REFERENCES public.forum_category(category_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: forum_thread forum_thread__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_thread
    ADD CONSTRAINT forum_thread__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: forum_thread forum_thread__page; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_thread
    ADD CONSTRAINT forum_thread__page FOREIGN KEY (page_id) REFERENCES public.page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: forum_thread forum_thread__post; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_thread
    ADD CONSTRAINT forum_thread__post FOREIGN KEY (last_post_id) REFERENCES public.forum_post(post_id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: forum_thread forum_thread__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.forum_thread
    ADD CONSTRAINT forum_thread__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: front_forum_feed front_forum_feed__page; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.front_forum_feed
    ADD CONSTRAINT front_forum_feed__page FOREIGN KEY (page_id) REFERENCES public.page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: front_forum_feed front_forum_feed__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.front_forum_feed
    ADD CONSTRAINT front_forum_feed__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: fts_entry fts_entry__forum_thread; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.fts_entry
    ADD CONSTRAINT fts_entry__forum_thread FOREIGN KEY (thread_id) REFERENCES public.forum_thread(thread_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: fts_entry fts_entry__page; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.fts_entry
    ADD CONSTRAINT fts_entry__page FOREIGN KEY (page_id) REFERENCES public.page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: fts_entry fts_entry__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.fts_entry
    ADD CONSTRAINT fts_entry__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: ip_block ip_block__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ip_block
    ADD CONSTRAINT ip_block__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: log_event log_event__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.log_event
    ADD CONSTRAINT log_event__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: member member__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.member
    ADD CONSTRAINT member__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: member member__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.member
    ADD CONSTRAINT member__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: member_application member_application__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.member_application
    ADD CONSTRAINT member_application__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: member_application member_application__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.member_application
    ADD CONSTRAINT member_application__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: member_invitation member_invitation__ozone_user__by_user_id; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.member_invitation
    ADD CONSTRAINT member_invitation__ozone_user__by_user_id FOREIGN KEY (by_user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: member_invitation member_invitation__ozone_user__user_id; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.member_invitation
    ADD CONSTRAINT member_invitation__ozone_user__user_id FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: member_invitation member_invitation__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.member_invitation
    ADD CONSTRAINT member_invitation__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: moderator moderator__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.moderator
    ADD CONSTRAINT moderator__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: moderator moderator__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.moderator
    ADD CONSTRAINT moderator__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: notification notification__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.notification
    ADD CONSTRAINT notification__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: ozone_session ozone_session__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ozone_session
    ADD CONSTRAINT ozone_session__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: page page__parent_page; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page
    ADD CONSTRAINT page__parent_page FOREIGN KEY (parent_page_id) REFERENCES public.page(page_id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: page page__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page
    ADD CONSTRAINT page__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: page_abuse_flag page_abuse_flag__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_abuse_flag
    ADD CONSTRAINT page_abuse_flag__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: page_abuse_flag page_abuse_flag__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_abuse_flag
    ADD CONSTRAINT page_abuse_flag__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: page_compiled page_compiled__page; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_compiled
    ADD CONSTRAINT page_compiled__page FOREIGN KEY (page_id) REFERENCES public.page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: page_edit_lock page_edit_lock__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_edit_lock
    ADD CONSTRAINT page_edit_lock__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: page_edit_lock page_edit_lock__page; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_edit_lock
    ADD CONSTRAINT page_edit_lock__page FOREIGN KEY (page_id) REFERENCES public.page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: page_inclusion page_inclusion__page__included_page_id; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_inclusion
    ADD CONSTRAINT page_inclusion__page__included_page_id FOREIGN KEY (included_page_id) REFERENCES public.page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: page_inclusion page_inclusion__page__including_page_id; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_inclusion
    ADD CONSTRAINT page_inclusion__page__including_page_id FOREIGN KEY (including_page_id) REFERENCES public.page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: page_link page_link__page__from_page_id; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_link
    ADD CONSTRAINT page_link__page__from_page_id FOREIGN KEY (from_page_id) REFERENCES public.page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: page_link page_link__page__to_page_id; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_link
    ADD CONSTRAINT page_link__page__to_page_id FOREIGN KEY (to_page_id) REFERENCES public.page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: page_rate_vote page_rate_vote__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_rate_vote
    ADD CONSTRAINT page_rate_vote__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: page_rate_vote page_rate_vote__page; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_rate_vote
    ADD CONSTRAINT page_rate_vote__page FOREIGN KEY (page_id) REFERENCES public.page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: page_tag page_tag__page; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.page_tag
    ADD CONSTRAINT page_tag__page FOREIGN KEY (page_id) REFERENCES public.page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: site_viewer page_viewer__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site_viewer
    ADD CONSTRAINT page_viewer__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: private_message private_message__ozone_user__from_user_id; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.private_message
    ADD CONSTRAINT private_message__ozone_user__from_user_id FOREIGN KEY (from_user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: private_message private_message__ozone_user__to_user_id; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.private_message
    ADD CONSTRAINT private_message__ozone_user__to_user_id FOREIGN KEY (to_user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: private_user_block private_user_block__ozone_user__blocked_user_id; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.private_user_block
    ADD CONSTRAINT private_user_block__ozone_user__blocked_user_id FOREIGN KEY (blocked_user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: private_user_block private_user_block__ozone_user__user_id; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.private_user_block
    ADD CONSTRAINT private_user_block__ozone_user__user_id FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: profile profile__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.profile
    ADD CONSTRAINT profile__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: simpletodo_list simpletedo_list__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.simpletodo_list
    ADD CONSTRAINT simpletedo_list__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: site_backup site_backup__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site_backup
    ADD CONSTRAINT site_backup__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: site_settings site_settings__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site_settings
    ADD CONSTRAINT site_settings__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: site_super_settings site_super_settings__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site_super_settings
    ADD CONSTRAINT site_super_settings__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: site_tag site_tag__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site_tag
    ADD CONSTRAINT site_tag__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: site_viewer site_viewer__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.site_viewer
    ADD CONSTRAINT site_viewer__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: theme theme__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.theme
    ADD CONSTRAINT theme__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: ucookie ucookie__ozone_session; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ucookie
    ADD CONSTRAINT ucookie__ozone_session FOREIGN KEY (session_id) REFERENCES public.ozone_session(session_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: ucookie ucookie__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.ucookie
    ADD CONSTRAINT ucookie__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: user_abuse_flag user_abuse_flag__ozone_user__target_user_id; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.user_abuse_flag
    ADD CONSTRAINT user_abuse_flag__ozone_user__target_user_id FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: user_abuse_flag user_abuse_flag__ozone_user__user_id; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.user_abuse_flag
    ADD CONSTRAINT user_abuse_flag__ozone_user__user_id FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: user_abuse_flag user_abuse_flag__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.user_abuse_flag
    ADD CONSTRAINT user_abuse_flag__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: user_block user_block__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.user_block
    ADD CONSTRAINT user_block__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: user_block user_block__site; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.user_block
    ADD CONSTRAINT user_block__site FOREIGN KEY (site_id) REFERENCES public.site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: user_settings user_settings__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.user_settings
    ADD CONSTRAINT user_settings__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: watched_forum_thread wached_forum_thread__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.watched_forum_thread
    ADD CONSTRAINT wached_forum_thread__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: watched_forum_thread watched_forum_thread__forum_thread; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.watched_forum_thread
    ADD CONSTRAINT watched_forum_thread__forum_thread FOREIGN KEY (thread_id) REFERENCES public.forum_thread(thread_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: watched_page watched_page__ozone_user; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.watched_page
    ADD CONSTRAINT watched_page__ozone_user FOREIGN KEY (user_id) REFERENCES public.ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: watched_page watched_page__page; Type: FK CONSTRAINT; Schema: public; Owner: wikijump
--

ALTER TABLE ONLY public.watched_page
    ADD CONSTRAINT watched_page__page FOREIGN KEY (page_id) REFERENCES public.page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: SCHEMA public; Type: ACL; Schema: -; Owner: postgres
--

GRANT ALL ON SCHEMA public TO wikijump;


--
-- PostgreSQL database dump complete }}}
--

--
-- PostgreSQL database cluster dump complete }}}
--

-- vim: set fdm=marker foldlevel=1:
