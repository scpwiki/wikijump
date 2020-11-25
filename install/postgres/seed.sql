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
1	4	1	t
2	5	6	t
3	6	1	t
4	7	9	t
5	8	9	t
\.


--
-- Data for Name: admin_notification; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.admin_notification (notification_id, site_id, body, type, viewed, date, extra, notify_online, notify_feed, notify_email) FROM stdin;
1	5	\N	NEW_MEMBER_APPLICATION	f	2020-06-02 22:35:59	\\x613a323a7b733a31343a226170706c69636174696f6e5f6964223b733a313a2231223b733a31323a2266726f6d5f757365725f6964223b733a313a2235223b7d	f	f	f
2	1	\N	NEW_MEMBER_APPLICATION	f	2020-06-03 05:43:02	\\x613a323a7b733a31343a226170706c69636174696f6e5f6964223b733a313a2232223b733a31323a2266726f6d5f757365725f6964223b733a313a2237223b7d	f	f	f
3	1	\N	NEW_MEMBER_APPLICATION	f	2020-06-08 03:15:53	\\x613a323a7b733a31343a226170706c69636174696f6e5f6964223b733a313a2233223b733a31323a2266726f6d5f757365725f6964223b733a323a223130223b7d	f	f	f
\.


--
-- Data for Name: anonymous_abuse_flag; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.anonymous_abuse_flag (flag_id, user_id, address, proxy, site_id, site_valid, global_valid) FROM stdin;
\.


--
-- Data for Name: category; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, enable_pingback_out, enable_pingback_in, autonumerate, page_title_template) FROM stdin;
6	2	nav	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
7	3	_default	t	20	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	f	1	\N	f	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
9	3	admin	f	21	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
11	3	nav	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
14	2	search	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
15	1	nav	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
2	2	_default	t	20	f	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	f	1	\N	f	nav:top	nav:side	\N	f	t	\N	\N	\N	t	f	f	\N
13	2	admin	f	21	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
17	2	forum	t	20	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
12	2	system	t	20	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
4	1	account	f	21	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
3	1	admin	f	21	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
16	1	search	t	20	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
5	1	user	f	21	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
8	3	profile	f	20	f	e:o;c:;m:;d:;a:;r:;z:;o:o	t	1	\N	f	nav:top	nav:profile-side	\N	\N	t	\N	\N	\N	t	f	f	\N
18	2	profile	t	20	t	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
19	1	system-all	t	20	t	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
20	1	system	t	20	t	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
21	1	auth	t	20	f	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	f	1	\N	f	nav:top	nav:side	\N	f	t	\N	\N	\N	t	f	f	\N
22	4	nav	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
23	4	search	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
24	4	_default	t	24	f	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	f	1	\N	f	nav:top	nav:side	\N	f	t	\N	\N	\N	t	f	f	\N
25	4	admin	f	21	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
26	4	system	t	20	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
46	1	info	t	27	t	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	https://www.FILEDOMAIN/local--theme/sigma9/style.css?1	t	f	f	\N
27	1	component	t	27	t	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N		t	f	f	\N
1	1	_default	t	27	f	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	f	1	\N	f	nav:top	nav:side	\N	f	t	\N	\N	https://www.FILEDOMAIN/local--theme/sigma9/style.css?1	t	f	f	\N
28	2	template	t	20	t	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
29	1	profile	t	27	t	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	https://www.FILEDOMAIN/local--theme/sigma9/style.css?1	t	f	f	\N
30	1	template	t	27	t	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	https://www.FILEDOMAIN/local--theme/sigma9/style.css?1	t	f	f	\N
31	3	template	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
32	5	nav	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
33	5	search	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
35	5	admin	f	21	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
36	5	system	t	20	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
37	5	template	t	20	t	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
59	8	forum	t	20	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
50	7	admin	t	21	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
49	7	_default	t	26	f	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	f	1	\N	f			\N	f	t	\N	\N	http://www.null.null	t	f	f	\N
53	8	nav	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
34	5	_default	t	28	f	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	f	1	\N	f	nav:top	nav:side	\N	f	t	\N	\N	\N	t	f	f	\N
38	6	nav	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
39	6	search	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
40	6	_default	t	20	f	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	f	1	\N	f	nav:top	nav:side	\N	f	t	\N	\N	\N	t	f	f	\N
41	6	admin	f	21	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
42	6	system	t	20	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
43	6	template	t	20	t	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
44	1	theme	t	27	t	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	https://www.FILEDOMAIN/local--theme/sigma9/style.css?1	t	f	f	\N
45	1	random	t	27	t	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	https://www.FILEDOMAIN/local--theme/sigma9/style.css?1	t	f	f	\N
54	8	search	t	20	t	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
55	8	_default	t	20	f	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	f	1	\N	f	nav:top	nav:side	\N	f	t	\N	\N	\N	t	f	f	\N
56	8	admin	f	21	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
57	8	system	t	20	f	v:arm;e:;c:;m:;d:;a:;r:;z:;o:	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
58	8	template	t	20	t	e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm	t	1	\N	t	nav:top	nav:side	\N	\N	t	\N	\N	\N	t	f	f	\N
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
1	76	1	badwikidot.png	image/png; charset=binary	PNG image data, 48 x 48, 8-bit/color RGBA, non-interlaced	PNG image data		901	2020-06-02 22:45:50	1	\N	f
2	76	1	black.png	image/png; charset=binary	PNG image data, 1 x 1, 8-bit/color RGB, non-interlaced	PNG image data		119	2020-06-02 22:47:47	1	\N	f
3	76	1	default.png	image/png; charset=binary	PNG image data, 13 x 13, 8-bit/color RGBA, non-interlaced	PNG image data		197	2020-06-02 22:47:56	1	\N	f
4	76	1	main.png	image/png; charset=binary	PNG image data, 13 x 13, 8-bit/color RGBA, non-interlaced	PNG image data		242	2020-06-02 22:48:10	1	\N	f
5	76	1	Twitter-icon.png	image/png; charset=binary	PNG image data, 30 x 30, 8-bit/color RGBA, non-interlaced	PNG image data		4364	2020-06-02 22:48:42	1	\N	f
6	76	1	Tumblr-icon.png	image/png; charset=binary	PNG image data, 30 x 30, 8-bit/color RGBA, non-interlaced	PNG image data		4478	2020-06-02 22:48:50	1	\N	f
7	76	1	tumblr2.png	image/png; charset=binary	PNG image data, 30 x 30, 8-bit colormap, non-interlaced	PNG image data		1665	2020-06-02 22:48:57	1	\N	f
8	76	1	series.png	image/png; charset=binary	PNG image data, 13 x 13, 8-bit/color RGBA, non-interlaced	PNG image data		257	2020-06-02 22:49:05	1	\N	f
9	76	1	Reddit-icon.png	image/png; charset=binary	PNG image data, 30 x 30, 8-bit/color RGBA, non-interlaced	PNG image data		4492	2020-06-02 22:49:14	1	\N	f
10	76	1	icon-Twitter-2020.png	image/png; charset=binary	PNG image data, 30 x 30, 8-bit/color RGBA, non-interlaced	PNG image data		1567	2020-06-02 22:49:30	1	\N	f
11	76	1	icon-Tumblr-2020.png	image/png; charset=binary	PNG image data, 30 x 30, 8-bit/color RGBA, non-interlaced	PNG image data		1537	2020-06-02 22:49:38	1	\N	f
12	76	1	icon-Instagram-2020.png	image/png; charset=binary	PNG image data, 30 x 30, 8-bit/color RGBA, non-interlaced	PNG image data		1984	2020-06-02 22:49:48	1	\N	f
13	76	1	icon-Reddit-2020.png	image/png; charset=binary	PNG image data, 30 x 30, 8-bit/color RGBA, non-interlaced	PNG image data		1619	2020-06-02 22:49:56	1	\N	f
14	76	1	icon-Facebook-2020.png	image/png; charset=binary	PNG image data, 30 x 30, 8-bit/color RGBA, non-interlaced	PNG image data		1546	2020-06-02 22:50:09	1	\N	f
15	76	1	icon-DeviantArt-2020.png	image/png; charset=binary	PNG image data, 30 x 30, 8-bit/color RGBA, non-interlaced	PNG image data		1524	2020-06-02 22:50:17	1	\N	f
16	76	1	home.png	image/png; charset=binary	PNG image data, 13 x 13, 8-bit/color RGBA, non-interlaced	PNG image data		251	2020-06-02 22:50:25	1	\N	f
17	76	1	help.png	image/png; charset=binary	PNG image data, 13 x 13, 8-bit/color RGBA, non-interlaced	PNG image data		289	2020-06-02 22:50:34	1	\N	f
18	76	1	FB-icon.png	image/png; charset=binary	PNG image data, 30 x 30, 8-bit/color RGBA, non-interlaced	PNG image data		4462	2020-06-02 22:50:43	1	\N	f
19	76	1	expand.png	image/png; charset=binary	PNG image data, 13 x 13, 8-bit/color RGBA, non-interlaced	PNG image data		283	2020-06-02 22:50:51	1	\N	f
20	76	1	DA-icon.png	image/png; charset=binary	PNG image data, 30 x 30, 8-bit/color RGBA, non-interlaced	PNG image data		1851	2020-06-02 22:51:07	1	\N	f
21	76	1	forum.png	image/png; charset=binary	PNG image data, 13 x 13, 8-bit/color RGBA, non-interlaced	PNG image data		253	2020-06-02 23:09:26	1	\N	f
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
1	1	Deleted threads	Deleted forum discussions should go here.	0	0	\N	t	t:;p:;e:;s:	\N	0	8	f
2	1	Per page discussions	This category groups discussions related to particular pages within this site.	0	1	\N	t	\N	\N	0	8	t
\.


--
-- Data for Name: forum_group; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.forum_group (group_id, name, description, sort_index, site_id, visible) FROM stdin;
1	Hidden	\N	0	8	f
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
8	t:m;p:m;e:o;s:	f	2
\.


--
-- Data for Name: forum_thread; Type: TABLE DATA; Schema: public; Owner: wikijump
--

COPY public.forum_thread (thread_id, user_id, user_string, category_id, title, description, number_posts, date_started, site_id, last_post_id, page_id, sticky, blocked) FROM stdin;
1	-1	\N	2	\N	\N	0	2020-06-12 07:28:36	8	\N	158	f	f
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
1	1	Manage	admin:manage	\N	1	\n\n\n	'admin':2C 'manag':1C,3C
2	2	You	account:you	\N	1	\n\n\n	'account':1C
3	3	Get a new wiki	new-site	\N	1	\n\nUse this simple form to create a new wiki.\nTo admins: you can customize this page by simply clicking "edit" at the bottom of the page.\n\n	'admin':18 'bottom':30 'click':26 'creat':13 'custom':21 'edit':27 'form':11 'get':1C 'new':3C,6C,15 'new-sit':5C 'page':23,33 'simpl':10 'simpli':25 'site':7C 'use':8 'wiki':4C,16
4	4	Info	user:info	\N	1	\n\n\n	'info':1C,3C 'user':2C
5	24	Search All Wikis	search:all	\N	1	\n\n\n	'search':1C,4C 'wiki':3C
6	33	Template	profile:template	\N	2	\n\nProfile has not been created (yet).\n	'creat':8 'profil':2C,4 'templat':1C,3C 'yet':9
7	36	Congratulations, welcome to your new wiki!	start	\N	2	\n\nIf this is your first site\nThen there are some things you need to know:\n\nYou can configure all security and other settings online, using the Site Manager. When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself. Check out the Permissions section.\nYour Wikidot site has two menus, one at the side called 'nav:side', and one at the top called 'nav:top'. These are Wikidot pages, and you can edit them like any page.\nTo edit a page, go to the page and click the Edit button at the bottom. You can change everything in the main area of your page. The Wikidot system is easy to learn and powerful.\nYou can attach images and other files to any page, then display them and link to them in the page.\nEvery Wikidot page has a history of edits, and you can undo anything. So feel secure, and experiment.\nTo start a forum on your site, see the Site Manager » Forum.\nThe license for this Wikidot site has been set to Creative Commons Attribution-Share Alike 3.0 License. If you want to change this, use the Site Manager.\nIf you want to learn more, make sure you visit the Documentation section at www.wikidot.org\n\nMore information about the Wikidot project can be found at www.wikidot.org.\n	'3.0':203 'access':50 'administr':59 'alik':202 'anyth':168 'area':123 'attach':138 'attribut':200 'attribution-shar':199 'bottom':115 'build':43 'button':112 'call':77,85 'chang':118,209 'check':62 'click':109 'common':198 'configur':25 'congratul':1C 'creativ':197 'display':147 'document':226 'easi':131 'edit':95,101,111,163 'everi':156 'everyth':119 'experi':173 'feel':170 'file':142 'first':12 'forum':177,186 'found':238 'go':104 'help':42 'histori':161 'imag':139 'inform':231 'invit':38 'know':22 'learn':133,219 'licens':188,204 'like':60,97 'link':150 'main':122 'make':57,221 'manag':35,54,184,214 'menus':72 'nav':78,86 'need':20 'new':5C 'one':73,81 'onlin':31 'page':91,99,103,107,126,145,155,158 'peopl':40 'permiss':65 'power':135 'project':235 'section':66,227 'secur':27,171 'see':181 'set':30,195 'share':201 'side':76,79 'site':13,34,45,53,69,180,183,192,213 'start':7C,175 'sure':222 'system':129 'thing':18 'top':84,87 'two':71 'undo':167 'unless':55 'use':32,211 'visit':224 'want':207,217 'welcom':2C 'wiki':6C 'wikidot':68,90,128,157,191,234 'www.wikidot.org':229,240 '�':185
8	37	List of all wikis	system-all:all-sites	\N	1	\n\nBelow is the list of public visible Wikis hosted at this service:\n\n	'all-sit':8C 'host':19 'list':1C,14 'public':16 'servic':22 'site':10C 'system':6C 'system-al':5C 'visibl':17 'wiki':4C,18
9	39	Search	system-all:search	\N	1	\n\n\nSearch all Wikis\nPerform a search through all public and visible wikis.\n\n\n\nSearch users\nTo look for someone, please enter:\n\nemail address of a person you are looking for (this will look for exact match)\nany part of the screen name or realname (lists all Users matching the query)\n\n\n\n	'address':27 'email':26 'enter':25 'exact':39 'list':49 'look':21,33,37 'match':40,52 'name':46 'part':42 'perform':9 'person':30 'pleas':24 'public':14 'queri':54 'realnam':48 'screen':45 'search':1C,5C,6,11,18 'someon':23 'system':3C 'system-al':2C 'user':19,51 'visibl':16 'wiki':8,17
10	48	Page Tags	system:page-tags	\N	1	\n\n\n\n\n	'page':1C,5C 'page-tag':4C 'system':3C 'tag':2C,6C
11	41	What Is A Wiki	what-is-a-wiki	\N	1	\n\nAccording to Wikipedia, the world largest wiki site:\n\nA Wiki ([ˈwiː.kiː] &lt;wee-kee&gt; or [ˈwɪ.kiː] &lt;wick-ey&gt;) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.\n\nAnd that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.\n	'accord':10 'add':40 'allow':37 'chang':46 'collabor':82 'communic':80 'content':48,77 'easili':52 'edit':44 'ey':30 'farm':62 'file':79 'great':69 'kee':24 'kiː':21,27 'largest':15 'otherwis':43 'part':59 'publish':76 'quick':50 'remov':41 'site':17,66 'tool':70 'type':33 'upload':78 'use':74 'user':38 'websit':35 'wee':23 'wee-ke':22 'what-is-a-wiki':5C 'wick':29 'wick-ey':28 'wiki':4C,9C,16,19,64 'wikipedia':12 'world':14 'ˈwiː':20 'ˈwɪ':26
12	42	How To Edit Pages	how-to-edit-pages	\N	1	\n\nIf you are allowed to edit pages in this Site, simply click on edit button at the bottom of the page. This will open an editor with a toolbar pallette with options.\nTo create a link to a new page, use syntax: [[[new page name]]] or [[[new page name | text to display]]]. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!\nAlthough creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit Documentation pages (at wikidot.org) to learn more.\n	'allow':13,98 'although':84 'bottom':27 'button':24 'click':21 'color':70 'creat':43,77,85,99 'differ':69 'display':61 'document':104 'easi':90 'edit':3C,8C,15,23,82,87 'editor':35 'exist':75 'follow':62 'how-to-edit-pag':5C 'learn':109 'link':45,64 'lot':94 'name':54,58 'new':48,52,56,79 'open':33 'option':41,96 'page':4C,9C,16,30,49,53,57,72,80,88,105 'pallett':39 'pleas':102 'power':100 'simpli':20 'site':19,101 'syntax':51 'text':59 'toolbar':38 'use':50 'visit':103 'wikidot.org':107
13	43	Wiki Members	system:members	\N	1	\n\nMembers:\n\n\nModerators\n\n\nAdmins\n\n	'admin':7 'member':2C,4C,5 'moder':6 'system':3C 'wiki':1C
14	44	How to join this wiki?	system:join	\N	1	\n\n\nPlease change this page according to your policy (configure first using Site Manager) and remove this note.\n\nWho can join?\nYou can write here who can become a member of this site.\nJoin!\nSo you want to become a member of this site? Tell us why and apply now!\n\n\nOr, if you already know a "secret password", go for it!\n\n	'accord':12 'alreadi':60 'appli':55 'becom':34,45 'chang':9 'configur':16 'first':17 'go':65 'join':3C,7C,27,40 'know':61 'manag':20 'member':36,47 'note':24 'page':11 'password':64 'pleas':8 'polici':15 'remov':22 'secret':63 'site':19,39,50 'system':6C 'tell':51 'us':52 'use':18 'want':43 'wiki':5C 'write':30
15	45	Recent changes	system:recent-changes	\N	1	\n\n\n	'chang':2C,6C 'recent':1C,5C 'recent-chang':4C 'system':3C
16	46	List all pages	system:list-all-pages	\N	1	\n\n\n	'list':1C,6C 'list-all-pag':5C 'page':3C,8C 'system':4C
17	47	Page Tags List	system:page-tags-list	\N	1	\n\n\n	'list':3C,8C 'page':1C,6C 'page-tags-list':5C 'system':4C 'tag':2C,7C
18	49	Log in	auth:login	\N	1	\n\n\n	'auth':2C 'log':1C 'login':3C
19	50	Create account - step 1	auth:newaccount	\N	1	\n\n\n	'1':4C 'account':2C 'auth':5C 'creat':1C 'newaccount':6C 'step':3C
20	51	Create account - step 2	auth:newaccount2	\N	1	\n\n\n	'2':4C 'account':2C 'auth':5C 'creat':1C 'newaccount2':6C 'step':3C
21	52	Create account - step 3	auth:newaccount3	\N	1	\n\n\n	'3':4C 'account':2C 'auth':5C 'creat':1C 'newaccount3':6C 'step':3C
22	69	Top	nav:top	\N	1	\n\n\nSample Menu\n\nExperienced users\nLink to a non-existing page\n\n\nEdit/Print\n\nEdit This Page\nPrint This Page\n\n\nAdmin\n\nEdit Top Navigation\nEdit Side Navigation\nSite Manager\n\n\n\n	'admin':20 'edit':14,21,24 'edit/print':13 'exist':11 'experienc':4 'link':6 'manag':28 'menu':3 'navig':23,26 'non':10 'non-exist':9 'page':12,16,19 'print':17 'sampl':2 'side':25 'site':27 'top':1A,22 'user':5
23	76	Side	nav:side	\N	1	\n\n\nWelcome page\n\n\nWhat is a Wiki?\nHow to edit pages?\nGet a new wiki!\n\nAll wikis\n\nRecent activity\nAll wikis\nWikis by tags\nSearch\n\nThis wiki\n\nHow to join this site?\nSite members\n\n\nRecent changes\nList all pages\nPage Tags\n\n\nSite Manager\n\nPage tags\n\n\nAdd a new page\n\n\nedit this panel\n	'activ':19 'add':46 'chang':36 'edit':10,50 'get':12 'join':30 'list':37 'manag':43 'member':34 'new':14,48 'page':3,11,39,40,44,49 'panel':52 'recent':18,35 'search':25 'side':1A 'site':32,33,42 'tag':24,41,45 'welcom':2 'wiki':7,15,17,21,22,27
24	72	Welcome to your Wikijump Custom Installation!	start	\N	1	\n\n\n\nCongratulations, you have successfully configured and launched your Wikidot custom installation!\nWhat to do next\nExperienced Wikidot users should start here.\nCustomize this wiki\nWikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!\n\nYou can configure all security and other settings online, using the Site Manager. When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself. Check out the Permissions section.\nYour Wikidot site has two menus, one at the side called 'nav:side', and one at the top called 'nav:top'. These are Wikidot pages, and you can edit them like any page.\nTo edit a page, go to the page and click the Edit button at the bottom. You can change everything in the main area of your page. The Wikidot system is easy to learn and powerful.\nYou can attach images and other files to any page, then display them and link to them in the page.\nEvery Wikidot page has a history of edits, and you can undo anything. So feel secure, and experiment.\nTo start a forum on your site, see the Site Manager » Forum.\nThe license for this Wikidot site has been set to Creative Commons Attribution-Share Alike 3.0 License. If you want to change this, use the Site Manager.\nIf you want to learn more, make sure you visit the Documentation section at www.wikidot.org\n\nCustomize the default templates\nThere are 2 initial default templates for other wikis. One is located at template-en and the other at template-blog. If someone creates a new wiki, these are presented as choices and the selected template is cloned to the new wiki's address. You should customize template-en and template-blog to suit your needs.\nCreate more templates\nSimply create new wikis with web site names starting with "template-" (e.g. "template-pl", "template-recipes") and your users will be have even more choices for the basic wiki layout they want to start with.\nVisit Wikidot.org\nGo to www.wikidot.org — home of the Wikidot open source software — for extra documentation, howtos, tips and support.\nVisit the Wikidot Community Site\nGo to community.wikidot.com — for even more tips, tricks and help from a very active community of Wikidot users.\nVisit the MyWikidot.local Project Site\nGo to my-wd-local.wikidot.com — for tips, discussions and how-to articles.\n\nMore information about the Wikidot project can be found at www.wikidot.org and the developers discussion at Wikidot dev-list.\nSearch all wikis\n\n\nSearch users\n\n	'2':262 '3.0':229 'access':77 'activ':397 'address':305 'administr':86 'alik':228 'anyth':195 'area':150 'articl':417 'attach':165 'attribut':226 'attribution-shar':225 'basic':352 'blog':282,315 'bottom':142 'build':70 'button':139 'call':104,112 'chang':145,235 'check':89 'choic':293,349 'click':136 'clone':299 'common':224 'communiti':382,398 'community.wikidot.com':386 'configur':11,52 'congratul':7 'consist':32 'creat':285,320,324 'creativ':223 'custom':5A,16,28,48,256,308 'default':258,264 'dev':436 'dev-list':435 'develop':431 'discuss':412,432 'display':174 'document':252,374 'e.g':334 'easi':158 'edit':122,128,138,190 'en':275,311 'even':347,388 'everi':183 'everyth':146 'experi':200 'experienc':22 'extra':373 'feel':197 'file':169 'forum':204,212 'found':426 'go':131,362,384,407 'help':69,393 'histori':188 'home':365 'how-to':414 'howto':375 'imag':166 'inform':419 'initi':263 'instal':6A,17 'invit':65 'launch':13 'layout':354 'learn':160,245 'licens':214,230 'like':87,124 'link':177 'list':437 'locat':271 'main':46,149 'make':84,247 'manag':62,81,211,240 'menus':99 'my-wd-local.wikidot.com':409 'mywikidot':4A 'mywikidot.local':404 'name':330 'nav':105,113 'need':319 'new':287,302,325 'next':21 'one':39,100,108,269 'onlin':58 'open':369 'page':118,126,130,134,153,172,182,185 'peopl':67 'permiss':92 'pl':337 'power':162 'present':291 'project':405,423 'recip':340 'right':40 'search':438,441 'section':93,253 'secur':54,198 'see':208 'select':296 'set':57,221 'sever':34 'share':227 'side':103,106 'simpli':323 'site':36,61,72,80,96,207,210,218,239,329,383,406 'softwar':371 'someon':284 'sourc':370 'start':26,202,331,358 'success':10 'suit':317 'support':378 'sure':248 'system':156 'templat':259,265,274,281,297,310,314,322,333,336,339 'template-blog':280,313 'template-en':273,309 'template-pl':335 'template-recip':338 'tip':376,390,411 'top':111,114 'trick':391 'two':98 'undo':194 'unless':82 'use':59,237 'user':24,343,401,442 'visit':250,360,379,402 'want':233,243,356 'web':328 'welcom':1A 'wiki':30,35,47,268,288,303,326,353,440 'wikidot':15,23,31,95,117,155,184,217,368,381,400,422,434 'wikidot.org':361 'www.wikidot.org':255,364,428
25	73	Wikijump Info	Wikijump-info	\N	1	\n\nJanuary 6, 2008\nThis installation was created using Remastersys Backup\nAs distributed, this is essentially "Wikidot-In-A-Box" - a fully functioning and configured installation of Wikidot v1 Open Source rev 393. You can use the normal "Get A New Wiki" process to create the following sites without any other configuration needed.\nSites\n\nmytest\nmyblog\nmysandbox\n\nYou can create new templates and sites by using the normal "Get A New Wiki" process, but you must then edit the /etc/hosts file so you can access the new sites. The easiest way to do this is using the Nautilus File Browser with root user access.\nHow to Edit The HOSTS File\n\nOpen a terminal window and type:\nsudo nautilus\nEnter your password.\n\nThe Nautilus File Browser opens with root user access to files.\n\nNavigate to the /etc folder and find the hosts file.\nRight-click and Open with "Text Editor"\nEdit the file so it resembles below, adding the names of your new templates or sites (this is your hosts file as it is distributed).\n\n\n127.0.0.1 localhost\n127.0.1.1 mywikidot\n# required for base install\n127.0.0.1 www.mywikidot.com\n127.0.0.1 profiles.mywikidot.com\n127.0.0.1 wikifiles.mywikidot.com\n127.0.0.1 template-en.mywikidot.com\n|# add new template sites if desired\n127.0.0.1 template-blog.mywikidot.com\n# add new site names here\n127.0.0.1 mytest.mywikidot.com\n127.0.0.1 myblog.mywikidot.com\n127.0.0.1 mysandbox.mywikidot.com\n\n# The following lines are desirable for IPv6 capable hosts\n::1 ip6-localhost ip6-loopback\nfe00::0 ip6-localnet\nff00::0 ip6-mcastprefix\nff02::1 ip6-allnodes\nff02::2 ip6-allrouters\nff02::3 ip6-allhosts\n\n\nSave the file.\n\nAs distributed, this hosts file will allow you to create and access 3 new sites: mytest.mywikidot.com, myblog.mywikidot.com and mysandbox.mywikidot.com without the need to edit the hosts file again. If you create new templates or sites, just edit the hosts file and add your new template(s) and site(s) to the list and you will be able to access them. You don't even have to restart any services or reopen your browser!\nOther Resources\nI have a Wikidot site dedicated to the installation, configuration and general tweaking of the open source version of Wikidot. Please visit MyWikidot.local for current information and more tips for using Wikidot on your own hardware or virtual machine.\nI have enjoyed putting this custom installation together and hope you enjoy using it!\n-Ed Johnson\n	'/etc':138 '/etc/hosts':82 '0':230,235 '1':222,240 '127.0.0.1':178,186,188,190,192,200,207,209,211 '127.0.1.1':180 '2':245 '2008':5 '3':250,269 '393':35 '6':4 'abl':313 'access':87,106,132,268,315 'ad':160 'add':194,202,298 'allhost':253 'allnod':243 'allow':263 'allrout':248 'backup':12 'base':184 'box':22 'browser':102,127,329 'capabl':220 'click':147 'configur':27,54,341 'creat':9,47,62,266,287 'current':356 'custom':376 'dedic':337 'desir':199,217 'distribut':14,177,258 'easiest':92 'ed':385 'edit':80,109,153,280,293 'editor':152 'enjoy':373,382 'enter':121 'essenti':17 'even':320 'fe00':229 'ff00':234 'ff02':239,244,249 'file':83,101,112,126,134,144,155,173,256,261,283,296 'find':141 'folder':139 'follow':49,214 'fulli':24 'function':25 'general':343 'get':41,71 'hardwar':367 'hope':380 'host':111,143,172,221,260,282,295 'info':2A 'inform':357 'instal':7,28,185,340,377 'ip6':224,227,232,237,242,247,252 'ip6-allhosts':251 'ip6-allnodes':241 'ip6-allrouters':246 'ip6-localhost':223 'ip6-localnet':231 'ip6-loopback':226 'ip6-mcastprefix':236 'ipv6':219 'januari':3 'johnson':386 'line':215 'list':308 'localhost':179,225 'localnet':233 'loopback':228 'machin':370 'mcastprefix':238 'must':78 'myblog':58 'myblog.mywikidot.com':210,273 'mysandbox':59 'mysandbox.mywikidot.com':212,275 'mytest':57 'mytest.mywikidot.com':208,272 'mywikidot':1A,181 'mywikidot.local':354 'name':162,205 'nautilus':100,120,125 'navig':135 'need':55,278 'new':43,63,73,89,165,195,203,270,288,300 'normal':40,70 'open':32,113,128,149,347 'password':123 'pleas':352 'process':45,75 'profiles.mywikidot.com':189 'put':374 'remastersi':11 'reopen':327 'requir':182 'resembl':158 'resourc':331 'restart':323 'rev':34 'right':146 'right-click':145 'root':104,130 'save':254 'servic':325 'site':50,56,66,90,168,197,204,271,291,304,336 'sourc':33,348 'sudo':119 'templat':64,166,196,289,301 'template-blog.mywikidot.com':201 'template-en.mywikidot.com':193 'termin':115 'text':151 'tip':360 'togeth':378 'tweak':344 'type':118 'use':10,38,68,98,362,383 'user':105,131 'v1':31 'version':349 'virtual':369 'visit':353 'way':93 'wiki':44,74 'wikidot':19,30,335,351,363 'wikidot-in-a-box':18 'wikifiles.mywikidot.com':191 'window':116 'without':51,276 'www.mywikidot.com':187
26	74	Top	nav:top	\N	2	\n\n\nSample Menu\n\nMyWikidot Home\nExperienced users\n\n\nEdit/Print\n\nEdit This Page\nPrint This Page\n\n\nAdmin\n\nEdit Top Navigation\nEdit Side Navigation\nSite Manager\n\n\n\n	'admin':15 'edit':9,16,19 'edit/print':8 'experienc':6 'home':5 'manag':23 'menu':3 'mywikidot':4 'navig':18,21 'page':11,14 'print':12 'sampl':2 'side':20 'site':22 'top':1A,17 'user':7
27	75	How To Edit Pages - Quickstart	how-to-edit-pages	\N	2	\n\nIf you are allowed to edit pages in this Site, simply click on edit button at the bottom of the page. This will open an editor with a toolbar pallette with options.\nTo create a link to a new page, use syntax: [[[new page name]]] or [[[new page name | text to display]]]. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!\nAlthough creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit Documentation pages (at wikidot.org) to learn more.\n	'allow':9,94 'although':80 'bottom':23 'button':20 'click':17 'color':66 'creat':39,73,81,95 'differ':65 'display':57 'document':100 'easi':86 'edit':3A,11,19,78,83 'editor':31 'exist':71 'follow':58 'learn':105 'link':41,60 'lot':90 'name':50,54 'new':44,48,52,75 'open':29 'option':37,92 'page':4A,12,26,45,49,53,68,76,84,101 'pallett':35 'pleas':98 'power':96 'quickstart':5A 'simpli':16 'site':15,97 'syntax':47 'text':55 'toolbar':34 'use':46 'visit':99 'wikidot.org':103
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
1	4	1	2008-12-06 17:49:31	t
2	5	6	2020-06-02 22:15:23	t
3	6	1	2020-06-06 18:59:55	t
4	1	7	2020-06-07 23:26:52	t
5	7	9	2020-06-08 03:06:18	t
6	1	10	2020-06-09 02:24:02	t
7	8	9	2020-06-11 16:17:16	t
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
1	Administrator	Admin	\N	admin@wikidot	admin	\N	\N	t	f	en
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
27	2	17	\N	31	30	28	0	Forum Categories	forum:start	2008-01-29 01:40:23	2008-01-29 01:40:23	1	\N	\N	1	f	0
28	2	17	\N	32	31	29	0	Forum Category	forum:category	2008-01-29 01:40:59	2008-01-29 01:40:59	1	\N	\N	1	f	0
29	2	17	\N	33	32	30	0	Forum Thread	forum:thread	2008-01-29 01:41:32	2008-01-29 01:41:32	1	\N	\N	1	f	0
30	2	17	\N	34	33	31	0	New Forum Thread	forum:new-thread	2008-01-29 01:42:10	2008-01-29 01:42:10	1	\N	\N	1	f	0
31	2	17	\N	35	34	32	0	Recent Forum Posts	forum:recent-posts	2008-01-29 01:42:42	2008-01-29 01:42:42	1	\N	\N	1	f	0
33	2	18	\N	37	36	34	0	Template	profile:template	2008-01-29 23:30:18	2008-01-29 23:30:18	1	\N	\N	1	f	0
36	2	2	\N	40	39	37	0	Congratulations, welcome to your new wiki!	start	2008-01-30 08:43:22	2008-01-30 08:43:22	1	\N	\N	1	f	0
37	1	19	\N	42	41	38	0	List of all wikis	system-all:all-sites	2008-01-30 08:54:56	2008-01-30 08:54:56	1	\N	\N	1	f	0
39	1	19	\N	46	45	41	0	Search	system-all:search	2008-01-30 09:07:05	2008-01-30 09:07:05	1	\N	\N	1	f	0
41	1	1	\N	51	50	45	0	What Is A Wiki	what-is-a-wiki	2008-01-30 16:11:56	2008-01-30 16:11:56	1	\N	\N	1	f	0
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
69	1	15	\N	223	170	75	4	Top	nav:top	2009-01-04 17:54:21	2020-06-06 18:51:56	1	\N	\N	1	f	0
72	1	1	\N	288	242	78	10	Welcome to your Wikijump Custom Installation!	start	2009-01-04 19:40:27	2020-06-09 02:22:09	1	\N	\N	1	f	0
73	1	1	\N	102	101	79	0	My Wikijump Info	wikijump-info	2009-01-04 19:46:55	2009-01-04 19:46:55	1	\N	\N	1	f	0
74	2	6	\N	103	102	80	0	Top	nav:top	2009-01-04 19:49:22	2009-01-04 19:49:22	1	\N	\N	1	f	0
75	2	2	\N	104	103	81	0	How To Edit Pages - Quickstart	how-to-edit-pages	2009-01-04 19:50:30	2009-01-04 19:50:30	1	\N	\N	1	f	0
76	1	15	\N	224	167	82	23	Side	nav:side	2009-01-04 19:51:32	2020-06-06 18:52:18	1	\N	\N	1	f	0
77	1	19	\N	106	105	83	0	List wikis by tags	system-all:sites-by-tags	2009-01-04 19:52:44	2009-01-04 19:52:44	1	\N	\N	1	f	0
78	1	19	\N	108	107	84	0	Activity across all wikis	system-all:activity	2009-01-04 19:54:26	2009-01-04 19:54:26	1	\N	\N	1	f	0
79	1	16	\N	109	108	85	0	Search Users	search:users	2009-01-04 19:55:43	2009-01-04 19:55:43	1	\N	\N	1	f	0
80	1	16	\N	110	109	86	0	Search This Wiki	search:site	2009-01-04 19:56:53	2009-01-04 19:56:53	1	\N	\N	1	f	0
87	1	27	\N	145	146	93	0	Theme	component:theme	2020-06-02 20:46:38	2020-06-02 20:46:38	1	\N	\N	1	f	0
88	2	28	\N	146	147	94	0	Profile	template:profile	2020-06-02 21:35:30	2020-06-02 21:35:30	1	\N	\N	1	f	0
89	1	29	\N	147	148	95	0	Template	profile:template	2020-06-02 21:38:10	2020-06-02 21:38:10	1	\N	\N	1	f	0
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
1	\n\nþmodule "managesite/ManageSiteModule"þ	2008-08-19 16:25:58
2	\n\nþmodule "account/AccountModule"þ	2008-08-19 16:25:58
3	\n\n<p>Use this simple form to create a new wiki.</p>\n<p>To admins: you can customize this page by simply clicking "edit" at the bottom of the page.</p>\nþmodule "newsite/NewSiteModule"þ	2020-06-08 02:10:26
4	\n\nþmodule "userinfo/UserInfoModule"þ	2008-08-19 16:25:58
5	\n\n<ul>\n<li><a href="/start">Welcome page</a></li>\n</ul>\n<ul>\n<li><a href="/what-is-a-wiki-site">What is a Wiki Site?</a></li>\n<li><a href="/how-to-edit-pages">How to edit pages?</a></li>\n</ul>\n<ul>\n<li><a href="/system:join">How to join this site?</a></li>\n<li><a href="/system:members">Site members</a></li>\n</ul>\n<ul>\n<li><a href="/system:recent-changes">Recent changes</a></li>\n<li><a href="/system:list-all-pages">List all pages</a></li>\n<li><a href="/system:page-tags-list">Page Tags</a></li>\n</ul>\n<ul>\n<li><a href="/admin:manage">Site Manager</a></li>\n</ul>\n<h2 id="toc0"><span>Page tags</span></h2>\nþmodule "wiki/pagestagcloud/PagesTagCloudModule" minFontSize%3D%2280%25%22+maxFontSize%3D%22200%25%22++maxColor%3D%228%2C8%2C64%22+minColor%3D%22100%2C100%2C128%22+target%3D%22system%3Apage-tags%22+limit%3D%2230%22 þ\n<h2 id="toc1"><span>Add a new page</span></h2>\nþmodule "misc/NewPageHelperModule" size%3D%2215%22+button%3D%22new+page%22 þ\n<p style="text-align: center;"><span style="font-size:80%;"><a href="/nav:side">edit this panel</a></span></p>\n	2008-01-30 08:39:25
6	\n\n<p>According to <a href="http://en.wikipedia.org/wiki/Wiki">Wikipedia</a>, the world largest wiki site:</p>\n<blockquote>\n<p>A <em>Wiki</em> ([ˈwiː.kiː] &lt;wee-kee&gt; or [ˈwɪ.kiː] &lt;wick-ey&gt;) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.</p>\n</blockquote>\n<p>And that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.</p>\n	2008-01-25 00:45:30
7	\n\n<p>Admin of this Wikidot installation.</p>\n	2008-01-25 01:05:59
8	\n\nþmodule "managesite/ManageSiteModule"þ	2008-01-25 01:06:39
10	\n\n<p>The profiles site is used to host user profiles. Each <tt>profile:username</tt> page contains a user-editable text that is included in the user's profile page.</p>\n<p>If you are viewing your own profile content page, feel free to edit it. You are the only one allowed to edit this page.</p>\n	2008-01-25 01:09:41
11	\n\n<p>The profiles site is used to host user profiles. Each <tt>profile:username</tt> page contains a user-editable text that is included in the user's profile page.</p>\n<ul>\n<li><a href="/start">Main page</a></li>\n<li><a href="/admin:manage">Manage this wiki</a></li>\n</ul>\n	2008-01-25 01:15:35
12	\n\n<p>The purpose of this wiki is to store user profiles.</p>\n	2008-01-25 01:15:35
14	\n\n<div class="wiki-note">\n<p>Please change this page according to your policy (configure first using <a href="/admin:manage">Site Manager</a>) and remove this note.</p>\n</div>\n<h1 id="toc0"><span>Who can join?</span></h1>\n<p>You can write here who can become a member of this site.</p>\n<h1 id="toc1"><span>Join!</span></h1>\n<p>So you want to become a member of this site? Tell us why and apply now!</p>\nþmodule "membership/MembershipApplyModule"þ<br />\n<p>Or, if you already know a "secret password", go for it!</p>\nþmodule "membership/MembershipByPasswordModule"þ	2008-01-29 00:57:39
15	\n\nþmodule "managesite/ManageSiteModule"þ	2008-01-29 00:57:39
16	\n\nþmodule "wiki/pagestagcloud/PagesTagCloudModule" limit%3D%22200%22+target%3D%22system%3Apage-tags%22 þ	2008-01-29 00:58:44
17	\n\nþmodule "changes/SiteChangesModule"þ	2008-01-29 00:59:15
18	\n\n<h1 id="toc0"><span>Members:</span></h1>\nþmodule "membership/MembersListModule"þ\n<h1 id="toc1"><span>Moderators</span></h1>\nþmodule "membership/MembersListModule" group%3D%22moderators%22 þ\n<h1 id="toc2"><span>Admins</span></h1>\nþmodule "membership/MembersListModule" group%3D%22admins%22 þ	2008-01-29 00:59:40
19	\n\nþmodule "search/SearchModule"þ	2008-01-29 01:01:49
20	\n\n<div style="float:right; width: 50%;">þmodule "wiki/pagestagcloud/PagesTagCloudModule" limit%3D%22200%22+target%3D%22system%3Apage-tags%22 þ</div>\nþmodule "wiki/pagestagcloud/PagesListByTagModule"þ	2008-01-29 01:03:43
21	\n\nþmodule "list/WikiPagesModule" preview%3D%22true%22 þ	2008-01-29 01:04:52
24	\n\nþmodule "search/SearchAllModule"þ	2008-08-19 16:25:58
27	\n\nþmodule "forum/ForumStartModule"þ	2008-01-29 01:40:24
28	\n\nþmodule "forum/ForumViewCategoryModule"þ	2008-01-29 01:40:59
29	\n\nþmodule "forum/ForumViewThreadModule"þ	2008-01-29 01:41:32
30	\n\nþmodule "forum/ForumNewThreadModule"þ	2008-01-29 01:42:10
31	\n\nþmodule "forum/ForumRecentPostsModule"þ	2008-01-29 01:42:42
33	\n\n<p>Profile has not been created (yet).</p>\n	2008-01-29 23:30:18
36	\n\n<h2 id="toc0"><span>If this is your first site</span></h2>\n<p>Then there are some things you need to know:</p>\n<ul>\n<li>You can configure all security and other settings online, using the <a href="/admin:manage">Site Manager</a>. When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself. Check out the <em>Permissions</em> section.</li>\n<li>Your Wikidot site has two menus, <a href="/nav:side">one at the side</a> called '<tt>nav:side</tt>', and <a href="/nav:top">one at the top</a> called '<tt>nav:top</tt>'. These are Wikidot pages, and you can edit them like any page.</li>\n<li>To edit a page, go to the page and click the <strong>Edit</strong> button at the bottom. You can change everything in the main area of your page. The Wikidot system is <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, '_blank'); return false;">easy to learn and powerful</a>.</li>\n<li>You can attach images and other files to any page, then display them and link to them in the page.</li>\n<li>Every Wikidot page has a history of edits, and you can undo anything. So feel secure, and experiment.</li>\n<li>To start a forum on your site, see the <a href="/admin:manage">Site Manager</a> » <em>Forum</em>.</li>\n<li>The license for this Wikidot site has been set to <a href="http://creativecommons.org/licenses/by-sa/3.0/" onclick="window.open(this.href, '_blank'); return false;">Creative Commons Attribution-Share Alike 3.0 License</a>. If you want to change this, use the Site Manager.</li>\n<li>If you want to learn more, make sure you visit the <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, '_blank'); return false;">Documentation section at www.wikidot.org</a></li>\n</ul>\n<p>More information about the Wikidot project can be found at <a href="http://www.wikidot.org" onclick="window.open(this.href, '_blank'); return false;">www.wikidot.org</a>.</p>\n	2008-01-30 08:43:22
37	\n\n<p>Below is the list of public visible Wikis hosted at this service:</p>\nþmodule "wiki/listallwikis/ListAllWikisModule"þ	2008-08-19 16:25:58
39	\n\n<div style="text-align: center;">\n<h1 id="toc0"><span>Search all Wikis</span></h1>\n<p>Perform a search through all public and visible wikis.</p>\nþmodule "search/SearchAllModule"þ\n<hr />\n<h1 id="toc1"><span>Search users</span></h1>\n<p>To look for someone, please enter:</p>\n<ul>\n<li>email address of a person you are looking for (this will look for exact match)</li>\n<li>any part of the screen name or realname (lists all Users matching the query)</li>\n</ul>\nþmodule "search/UserSearchModule"þ</div>\n	2008-08-19 16:25:59
41	\n\n<p>According to <a href="http://en.wikipedia.org/wiki/Wiki">Wikipedia</a>, the world largest wiki site:</p>\n<blockquote>\n<p>A <em>Wiki</em> ([ˈwiː.kiː] &lt;wee-kee&gt; or [ˈwɪ.kiː] &lt;wick-ey&gt;) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.</p>\n</blockquote>\n<p>And that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.</p>\n	2020-06-08 02:10:26
42	\n\n<p>If you are allowed to edit pages in this Site, simply click on <em>edit</em> button at the bottom of the page. This will open an editor with a toolbar pallette with options.</p>\n<p>To create a link to a new page, use syntax: <tt>[[[new page name]]]</tt> or <tt>[[[new page name | text to display]]]</tt>. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!</p>\n<p>Although creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, '_blank'); return false;">Documentation pages</a> (at wikidot.org) to learn more.</p>\n	2020-06-08 02:10:26
43	\n\n<h1 id="toc0"><span>Members:</span></h1>\nþmodule "membership/MembersListModule"þ\n<h1 id="toc1"><span>Moderators</span></h1>\nþmodule "membership/MembersListModule" group%3D%22moderators%22 þ\n<h1 id="toc2"><span>Admins</span></h1>\nþmodule "membership/MembersListModule" group%3D%22admins%22 þ	2008-08-19 16:25:59
44	\n\n<div class="wiki-note">\n<p>Please change this page according to your policy (configure first using <a href="/admin:manage">Site Manager</a>) and remove this note.</p>\n</div>\n<h1 id="toc0"><span>Who can join?</span></h1>\n<p>You can write here who can become a member of this site.</p>\n<h1 id="toc1"><span>Join!</span></h1>\n<p>So you want to become a member of this site? Tell us why and apply now!</p>\nþmodule "membership/MembershipApplyModule"þ\n<p>Or, if you already know a "secret password", go for it!</p>\nþmodule "membership/MembershipByPasswordModule"þ	2008-08-19 16:25:59
45	\n\nþmodule "changes/SiteChangesModule"þ	2008-08-19 16:25:59
46	\n\nþmodule "list/WikiPagesModule" preview%3D%22true%22 þ	2008-08-19 16:25:59
47	\n\nþmodule "wiki/pagestagcloud/PagesTagCloudModule" limit%3D%22200%22+target%3D%22system%3Apage-tags%22 þ	2008-08-19 16:25:59
48	\n\n<div style="float:right; width: 50%;">þmodule "wiki/pagestagcloud/PagesTagCloudModule" limit%3D%22200%22+target%3D%22system%3Apage-tags%22 þ</div>\nþmodule "wiki/pagestagcloud/PagesListByTagModule"þ	2008-08-19 16:25:59
49	\n\nþmodule "login/LoginModule"þ	2008-08-19 16:25:59
50	\n\nþmodule "createaccount2/CreateAccountModule"þ	2008-08-19 16:25:59
51	\n\nþmodule "createaccount2/CreateAccount2Module"þ	2008-08-19 16:25:59
52	\n\nþmodule "createaccount2/CreateAccount3Module"þ	2008-08-19 16:25:59
69	\n\n<ul>\n<li><a href="javascript:;">Sample Menu</a>\n<ul>\n<li><a href="/mywikidot-info">Experienced users</a></li>\n<li><a class="newpage" href="/mywikidot-blank">Link to a non-existing page</a></li>\n</ul>\n</li>\n<li><a href="javascript:;">Edit/Print</a>\n<ul>\n<li><a class="wiki-standalone-button" href="javascript:;" onclick="WIKIDOT.page.listeners.editClick(event)">Edit This Page</a></li>\n<li><a class="wiki-standalone-button" href="javascript:;" onclick="WIKIDOT.page.listeners.printClick(event)">Print This Page</a></li>\n</ul>\n</li>\n<li><a href="javascript:;">Admin</a>\n<ul>\n<li><a href="/nav:top">Edit Top Navigation</a></li>\n<li><a href="/nav:side">Edit Side Navigation</a></li>\n<li><a href="/admin:manage">Site Manager</a></li>\n</ul>\n</li>\n</ul>\n	2020-06-06 18:51:56
72	\n\nþmodule "newmodules/css/CSSModule" +module_body%3D%22%2523page-title%2B%257B%2Bcolor%253A%2Bgreen%253B%2B%257D%22 þ\n<p>Congratulations, you have successfully configured and launched your Wikidot custom installation!</p>\n<h1 id="toc0"><span>What to do next</span></h1>\n<h2 id="toc1"><span>Experienced Wikidot users should <a href="/mywikidot-info">start here</a>.</span></h2>\n<h2 id="toc2"><span>Customize this wiki</span></h2>\n<p>Wikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!</p>\n<ul>\n<li>You can configure all security and other settings online, using the <a href="/admin:manage">Site Manager</a>. When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself. Check out the <em>Permissions</em> section.</li>\n<li>Your Wikidot site has two menus, <a href="/nav:side">one at the side</a> called '<tt>nav:side</tt>', and <a href="/nav:top">one at the top</a> called '<tt>nav:top</tt>'. These are Wikidot pages, and you can edit them like any page.</li>\n<li>To edit a page, go to the page and click the <strong>Edit</strong> button at the bottom. You can change everything in the main area of your page. The Wikidot system is <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, '_blank'); return false;">easy to learn and powerful</a>.</li>\n<li>You can attach images and other files to any page, then display them and link to them in the page.</li>\n<li>Every Wikidot page has a history of edits, and you can undo anything. So feel secure, and experiment.</li>\n<li>To start a forum on your site, see the <a href="/admin:manage">Site Manager</a> » <em>Forum</em>.</li>\n<li>The license for this Wikidot site has been set to <a href="http://creativecommons.org/licenses/by-sa/3.0/" onclick="window.open(this.href, '_blank'); return false;">Creative Commons Attribution-Share Alike 3.0 License</a>. If you want to change this, use the Site Manager.</li>\n<li>If you want to learn more, make sure you visit the <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, '_blank'); return false;">Documentation section at www.wikidot.org</a></li>\n</ul>\n<h2 id="toc3"><span>Customize the default templates</span></h2>\n<p>There are 2 initial default templates for other wikis. One is located at <a href="http://template-en.wikidork.com/start">template-en</a> and the other at <a href="http://template-blog.wikidork.com/start">template-blog</a>. If someone creates a new wiki, these are presented as choices and the selected template is cloned to the new wiki's address. You should customize <a href="http://template-en.wikidork.com/start">template-en</a> and <a href="http://template-blog.wikidork.com/start">template-blog</a> to suit your needs.</p>\n<h2 id="toc4"><span>Create more templates</span></h2>\n<p>Simply create new wikis with <strong>web site names</strong> starting with "template-" (e.g. "template-pl", "template-recipes") and your users will be have even more choices for the basic wiki layout they want to start with.</p>\n<h2 id="toc5"><span>Visit Wikidot.org</span></h2>\n<p>Go to <strong><a href="http://www.wikidot.org" onclick="window.open(this.href, '_blank'); return false;">www.wikidot.org</a></strong> — home of the Wikidot open source software — for extra documentation, howtos, tips and support.</p>\n<h2 id="toc6"><span>Visit the Wikidot Community Site</span></h2>\n<p>Go to <strong><a href="http://community.wikidot.com" onclick="window.open(this.href, '_blank'); return false;">community.wikidot.com</a></strong> — for even more tips, tricks and help from a very active community of Wikidot users.</p>\n<h2 id="toc7"><span>Visit the MyWikidot.local Project Site</span></h2>\n<p>Go to <strong><a href="http://my-wd-local.wikidot.com/" onclick="window.open(this.href, '_blank'); return false;">my-wd-local.wikidot.com</a></strong> — for tips, discussions and how-to articles.</p>\n<hr />\n<p>More information about the Wikidot project can be found at <a href="http://www.wikidot.org" onclick="window.open(this.href, '_blank'); return false;">www.wikidot.org</a> and the developers discussion at <a href="http://groups.google.com/group/wikidot" onclick="window.open(this.href, '_blank'); return false;">Wikidot dev-list</a>.</p>\n<h1 id="toc8"><span>Search all wikis</span></h1>\nþmodule "search/SearchAllModule"þ\n<h1 id="toc9"><span>Search users</span></h1>\nþmodule "search/UserSearchModule"þ	2020-06-09 02:22:09
73	\n\n<h3 id="toc0"><span>January 6, 2008</span></h3>\n<p>This installation was created using <a href="http://www.remastersys.klikit-linux.com/" onclick="window.open(this.href, '_blank'); return false;">Remastersys Backup</a></p>\n<p>As distributed, this is essentially "<strong>Wikidot-In-A-Box</strong>" - a fully functioning and configured installation of <strong>Wikidot v1 Open Source rev 393</strong>. You can use the normal "<em>Get A New Wiki</em>" process to create the following sites without any other configuration needed.</p>\n<h2 id="toc1"><span>Sites</span></h2>\n<ul>\n<li>mytest</li>\n<li>myblog</li>\n<li>mysandbox</li>\n</ul>\n<p>You can create new templates and sites by using the normal "<em>Get A New Wiki</em>" process, but you must then edit the <em><strong>/etc/hosts</strong></em> file so you can access the new sites. The easiest way to do this is using the <strong>Nautilus File Browser</strong> with <em>root</em> user access.</p>\n<h2 id="toc2"><span>How to Edit The HOSTS File</span></h2>\n<ul>\n<li>Open a terminal window and type:</li>\n<li>sudo nautilus</li>\n<li>Enter your password.</li>\n</ul>\n<p>The Nautilus File Browser opens with root user access to files.</p>\n<ul>\n<li>Navigate to the <strong>/etc</strong> folder and find the <strong>hosts</strong> file.</li>\n<li>Right-click and <strong>Open with "Text Editor"</strong></li>\n<li>Edit the file so it resembles below, adding the names of your new templates or sites (this is your hosts file as it is distributed).</li>\n</ul>\n<blockquote>\n<p>127.0.0.1 localhost<br />\n127.0.1.1 mywikidot<br />\n# required for base install<br />\n<span style="color: red;">127.0.0.1 www.mywikidot.com</span><br />\n<span style="color: red;">127.0.0.1 profiles.mywikidot.com</span><br />\n<span style="color: red;">127.0.0.1 wikifiles.mywikidot.com</span><br />\n<span style="color: red;">127.0.0.1 template-en.mywikidot.com</span><br />\n|# add new template sites if desired<br />\n<span style="color: red;">127.0.0.1 template-blog.mywikidot.com</span><br />\n# add new site names here<br />\n<span style="color: red;">127.0.0.1 mytest.mywikidot.com</span><br />\n<span style="color: red;">127.0.0.1 myblog.mywikidot.com</span><br />\n<span style="color: red;">127.0.0.1 mysandbox.mywikidot.com</span><br />\n<br />\n# The following lines are desirable for IPv6 capable hosts<br />\n::1 ip6-localhost ip6-loopback<br />\nfe00::0 ip6-localnet<br />\nff00::0 ip6-mcastprefix<br />\nff02::1 ip6-allnodes<br />\nff02::2 ip6-allrouters<br />\nff02::3 ip6-allhosts</p>\n</blockquote>\n<ul>\n<li>Save the file.</li>\n</ul>\n<p>As distributed, this hosts file will allow you to create and access 3 new sites: <strong>mytest.mywikidot.com</strong>, <strong>myblog.mywikidot.com</strong> and <strong>mysandbox.mywikidot.com</strong> without the need to edit the hosts file again. If you create new templates or sites, just edit the hosts file and add your new template(s) and site(s) to the list and you will be able to access them. You don't even have to restart any services or reopen your browser!</p>\n<h2 id="toc3"><span>Other Resources</span></h2>\n<p>I have a Wikidot site dedicated to the installation, configuration and general tweaking of the open source version of Wikidot. Please visit <span style="font-size:125%;"><a href="http://my-wd-local.wikidot.com/" onclick="window.open(this.href, '_blank'); return false;">MyWikidot.local</a></span> for current information and more tips for using Wikidot on your own hardware or virtual machine.</p>\n<p>I have enjoyed putting this custom installation together and hope you enjoy using it!<br />\n-Ed Johnson</p>\n	2020-06-08 02:10:26
74	\n\n<ul>\n<li><a href="javascript:;">Sample Menu</a>\n<ul>\n<li><a href="http://www.mywikidot.com/start">MyWikidot Home</a></li>\n<li><a href="http://www.mywikidot.com/mywikidot-info">Experienced users</a></li>\n</ul>\n</li>\n<li><a href="javascript:;">Edit/Print</a>\n<ul>\n<li><a class="wiki-standalone-button" href="javascript:;" onclick="WIKIDOT.page.listeners.editClick(event)">Edit This Page</a></li>\n<li><a class="wiki-standalone-button" href="javascript:;" onclick="WIKIDOT.page.listeners.printClick(event)">Print This Page</a></li>\n</ul>\n</li>\n<li><a href="javascript:;">Admin</a>\n<ul>\n<li><a href="/nav:top">Edit Top Navigation</a></li>\n<li><a href="/nav:side">Edit Side Navigation</a></li>\n<li><a href="/admin:manage">Site Manager</a></li>\n</ul>\n</li>\n</ul>\n	2009-01-04 19:49:22
75	\n\n<p>If you are allowed to edit pages in this Site, simply click on <em>edit</em> button at the bottom of the page. This will open an editor with a toolbar pallette with options.</p>\n<p>To create a link to a new page, use syntax: <tt>[[[new page name]]]</tt> or <tt>[[[new page name | text to display]]]</tt>. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!</p>\n<p>Although creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, '_blank'); return false;">Documentation pages</a> (at wikidot.org) to learn more.</p>\n	2009-01-04 19:50:30
76	\n\n<ul>\n<li><a href="/start">Welcome page</a></li>\n</ul>\n<ul>\n<li><a href="/what-is-a-wiki">What is a Wiki?</a></li>\n<li><a href="/how-to-edit-pages">How to edit pages?</a></li>\n<li><a href="/new-site">Get a new wiki!</a></li>\n</ul>\n<h1 id="toc0"><span>All wikis</span></h1>\n<ul>\n<li><a href="/system-all:activity">Recent activity</a></li>\n<li><a href="/system-all:all-sites">All wikis</a></li>\n<li><a href="/system-all:sites-by-tags">Wikis by tags</a></li>\n<li><a href="/system-all:search">Search</a></li>\n</ul>\n<h1 id="toc1"><span>This wiki</span></h1>\n<ul>\n<li><a href="/system:join">How to join this site?</a></li>\n<li><a href="/system:members">Site members</a></li>\n</ul>\n<ul>\n<li><a href="/system:recent-changes">Recent changes</a></li>\n<li><a href="/system:list-all-pages">List all pages</a></li>\n<li><a href="/system:page-tags-list">Page Tags</a></li>\n</ul>\n<ul>\n<li><a href="/admin:manage">Site Manager</a></li>\n</ul>\n<h2 id="toc2"><span>Page tags</span></h2>\nþmodule "wiki/pagestagcloud/PagesTagCloudModule" minFontSize%3D%2280%25%22+maxFontSize%3D%22200%25%22++maxColor%3D%228%2C8%2C64%22+minColor%3D%22100%2C100%2C128%22+target%3D%22system%3Apage-tags%22+limit%3D%2230%22 þ\n<h2 id="toc3"><span>Add a new page</span></h2>\nþmodule "misc/NewPageHelperModule" size%3D%2215%22+button%3D%22new+page%22 þ\n<p style="text-align: center;"><span style="font-size:80%;"><a href="/nav:side">edit this panel</a></span></p>\n	2020-06-06 18:52:18
77	\n\nþmodule "wiki/sitestagcloud/SitesTagCloudModule" limit%3D%22100%22+target%3D%22system-all%3Asites-by-tags%22 þþmodule "wiki/sitestagcloud/SitesListByTagModule"þ	2009-01-04 19:52:44
78	\n\n<table>\n<tr>\n<td style="width: 45%; padding-right: 2%; border-right: 1px solid #999; vertical-align:top;">\n<h2 id="toc0"><span>Recent edits (all wikis)</span></h2>\nþmodule "wiki/sitesactivity/RecentWPageRevisionsModule"þ</td>\n<td style="width: 45%; padding-left: 2%; vertical-align:top;">\n<h2 id="toc1"><span>Top Sites</span></h2>\nþmodule "wiki/sitesactivity/MostActiveSitesModule"þ\n<h2 id="toc2"><span>Top Forums</span></h2>\nþmodule "wiki/sitesactivity/MostActiveForumsModule"þ\n<h2 id="toc3"><span>New users</span></h2>\nþmodule "wiki/sitesactivity/NewWUsersModule"þ\n<h2 id="toc4"><span>Some statistics</span></h2>\nþmodule "wiki/sitesactivity/SomeGlobalStatsModule"þ</td>\n</tr>\n</table>\n	2009-01-04 19:54:27
79	\n\n<p>To look for someone, please enter:</p>\n<ul>\n<li>email address of a person you are looking for (this will look for exact match)</li>\n<li>any part of the screen name or realname (lists all Users matching the query)</li>\n</ul>\nþmodule "search/UserSearchModule"þ	2009-01-04 19:55:44
80	\n\nþmodule "search/SearchModule"þ	2009-01-04 19:56:53
87	\n\n	2020-06-02 20:46:38
88	\n\n<p>No profile has been set up yet for this user.</p>\n	2020-06-02 21:35:30
89	\n\n<p>No profile has yet been set up.</p>\n	2020-06-02 21:38:10
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
1	1	23	http://www.wikidot.org/doc	f	\N	2008-12-06 17:22:05
2	1	23	http://creativecommons.org/licenses/by-sa/3.0/	f	\N	2008-12-06 17:22:05
3	1	23	http://www.wikidot.org	f	\N	2008-12-06 17:22:05
4	1	23	http://community.wikidot.com	f	\N	2008-12-06 17:22:05
5	4	56	http://en.wikipedia.org/wiki/Wiki	f	\N	2008-12-06 17:49:33
6	4	57	http://www.wikidot.org/doc	f	\N	2008-12-06 17:49:33
7	4	57	http://creativecommons.org/licenses/by-sa/3.0/	f	\N	2008-12-06 17:49:33
8	4	57	http://www.wikidot.org	f	\N	2008-12-06 17:49:33
9	4	58	http://www.wikidot.org/doc	f	\N	2008-12-06 17:49:33
10	1	66	http://www.virtualbox.org/	f	\N	2008-12-06 20:18:57
11	1	66	http://my-wd-local.wikidot.com/	f	\N	2008-12-06 20:35:39
14	1	67	http://www.remastersys.klikit-linux.com/	f	\N	2009-01-04 17:30:30
15	1	23	http://my-wd-local.wikidot.com/	f	\N	2009-01-04 18:05:20
16	1	70	http://www.wikidot.org/doc	f	\N	2009-01-04 18:06:40
17	1	70	http://creativecommons.org/licenses/by-sa/3.0/	f	\N	2009-01-04 18:06:40
18	1	70	http://www.wikidot.org	f	\N	2009-01-04 18:06:40
19	1	70	http://community.wikidot.com	f	\N	2009-01-04 18:06:40
20	1	70	http://my-wd-local.wikidot.com/	f	\N	2009-01-04 18:06:40
21	1	71	http://www.wikidot.org/doc	f	\N	2009-01-04 18:09:08
24	1	71	http://community.wikidot.com	f	\N	2009-01-04 18:09:08
28	1	72	http://creativecommons.org/licenses/by-sa/3.0/	f	\N	2009-01-04 19:40:27
33	1	73	http://www.remastersys.klikit-linux.com/	f	\N	2009-01-04 19:46:55
36	5	97	http://en.wikipedia.org/wiki/Wiki	f	\N	2020-06-02 22:15:24
37	5	98	http://www.wikidot.org/doc	f	\N	2020-06-02 22:15:24
39	5	98	http://www.wikidot.org	f	\N	2020-06-02 22:15:24
69	5	95	http://fondazionescp.wikidot.com	f	\N	2020-06-02 22:59:32
72	5	95	http://scp-cs.wikidot.com	f	\N	2020-06-02 22:59:32
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
16	5	14	\N	2
18	14	15	\N	2
19	5	15	\N	2
20	5	16	\N	2
21	5	17	\N	2
22	5	18	\N	2
23	5	21	\N	2
44	36	15	\N	2
45	36	5	\N	2
64	44	1	\N	1
68	53	57	\N	4
69	53	56	\N	4
70	53	58	\N	4
71	53	60	\N	4
72	53	63	\N	4
73	53	62	\N	4
74	53	65	\N	4
75	53	61	\N	4
76	53	59	\N	4
77	53	53	\N	4
80	57	59	\N	4
81	57	53	\N	4
82	57	54	\N	4
83	60	59	\N	4
84	54	60	\N	4
85	54	54	\N	4
86	54	53	\N	4
87	54	59	\N	4
91	53	68	\N	4
364	110	114	\N	6
97	69	1	\N	1
365	110	113	\N	6
366	110	115	\N	6
367	110	117	\N	6
114	74	74	\N	2
115	74	5	\N	2
116	74	15	\N	2
368	110	120	\N	6
369	110	119	\N	6
370	110	122	\N	6
371	110	118	\N	6
372	110	116	\N	6
373	110	110	\N	6
125	76	44	\N	1
374	111	111	\N	6
127	76	45	\N	1
375	111	110	\N	6
376	111	116	\N	6
377	114	116	\N	6
378	114	110	\N	6
379	114	111	\N	6
380	117	116	\N	6
382	125	\N	lowest-rated-scps	1
135	94	101	\N	5
383	125	\N	lowest-rated-articles	1
137	94	103	\N	5
386	132	130	\N	1
387	132	131	\N	1
388	132	\N	jonathan-ball-s-proposal	1
389	132	\N	dr-gears-s-proposal	1
390	132	\N	dr-clef-s-proposal	1
391	132	\N	qntm-s-proposal	1
144	95	100	\N	5
145	98	100	\N	5
146	98	94	\N	5
147	98	95	\N	5
148	101	100	\N	5
392	132	\N	scp-001-o5	1
393	132	\N	dr-manns-proposal	1
394	132	\N	mackenzie-s-proposal	1
395	132	\N	sandrewswann-s-proposal	1
396	132	\N	scantron-s-proposal	1
397	132	\N	djoric-dmatix-proposal	1
398	132	\N	roget-s-proposal	1
399	132	\N	ouroboros	1
400	132	\N	kate-mctiriss-s-proposal	1
401	132	\N	kalinins-proposal	1
402	132	\N	wrong-proposal	1
403	132	\N	shaggydredlocks-proposal	1
404	132	\N	spikebrennan-s-proposal	1
405	132	\N	wjs-proposal	1
406	132	\N	billiths-proposal	1
407	132	\N	tanhony-s-proposal	1
408	132	\N	lily-s-proposal	1
409	132	\N	tuftos-proposal	1
410	132	\N	jim-north-s-proposal	1
411	132	\N	i-h-p-proposal	1
412	132	\N	scp-001-ex	1
413	132	\N	captain-kirby-s-proposal	1
414	132	\N	jack-ike-s-proposal-i	1
415	132	\N	jack-ike-s-proposal-ii	1
416	132	\N	tanhony-s-proposal-ii	1
434	154	158	\N	8
435	154	157	\N	8
436	154	159	\N	8
437	154	161	\N	8
438	154	164	\N	8
439	154	163	\N	8
440	154	166	\N	8
441	154	162	\N	8
442	154	160	\N	8
443	154	154	\N	8
444	155	155	\N	8
445	155	154	\N	8
446	155	160	\N	8
450	161	160	\N	8
452	158	168	\N	8
381	124	\N	most-recently-created	1
384	127	\N	image-use-policy	1
385	127	\N	chat-guide	1
238	94	\N	scp-series	5
239	94	\N	scp-series-2	5
240	94	\N	scp-series-3	5
241	94	\N	scp-series-4	5
242	94	\N	scp-series-5	5
243	94	\N	scp-series-6	5
244	94	\N	scp-series-1-tales-edition	5
245	94	\N	scp-series-2-tales-edition	5
246	94	\N	scp-series-3-tales-edition	5
247	94	\N	scp-series-4-tales-edition	5
248	94	\N	scp-series-5-tales-edition	5
249	94	\N	foundation-tales	5
250	94	\N	canon-hub	5
251	94	\N	scp-international	5
252	94	\N	goi-formats	5
253	94	\N	scp-ex	5
254	94	\N	top-rated-pages-this-month	5
255	94	\N	new-pages-feed	5
256	94	\N	random:random-scp	5
257	94	\N	random:random-tale	5
258	94	\N	http:www-scp-wiki-net-most-recently-edited	5
259	94	\N	lowest-rated-pages	5
260	94	\N	guide-hub	5
261	94	\N	contribute	5
262	94	\N	http:www-scp-wiki-net-young-and-under-30	5
263	94	\N	seminars-hub	5
264	94	\N	site-rules	5
265	94	\N	forum:start	5
266	94	\N	forum:recent-posts	5
267	94	\N	chat-guide	5
268	94	\N	authors-pages	5
269	94	\N	news	5
270	94	\N	http:05command-wikidot-com-staff-policy-hub	5
271	94	\N	how-to-write-an-scp	5
272	94	\N	tag-search	5
273	94	\N	usertools	5
274	94	\N	sandbox	5
275	94	\N	contact-staff	5
276	95	\N	scp-series-6	5
277	95	\N	scp-series-5	5
278	95	\N	scp-series-5-tales-edition	5
279	95	\N	scp-series-4	5
280	95	\N	scp-series-4-tales-edition	5
281	95	\N	scp-series-3	5
282	95	\N	scp-series-3-tales-edition	5
283	95	\N	scp-series-2	5
284	95	\N	scp-series-2-tales-edition	5
285	95	\N	scp-series	5
286	95	\N	scp-series-1-tales-edition	5
287	95	\N	foundation-tales	5
288	95	\N	series-archive	5
289	95	\N	incident-reports-eye-witness-interviews-and-personal-logs	5
290	95	\N	creepy-pasta	5
291	95	\N	user-curated-lists	5
292	95	\N	joke-scps	5
293	95	\N	joke-scps-tales-edition	5
294	95	\N	scp-ex	5
295	95	\N	explained-scps-tales-edition	5
296	95	\N	goi-formats	5
297	95	\N	audio-adaptations	5
298	95	\N	scp-artwork-hub	5
299	95	\N	contest-archive	5
300	95	\N	canon-hub	5
301	95	\N	groups-of-interest	5
302	95	\N	log-of-anomalous-items	5
303	95	\N	log-of-extranormal-events	5
304	95	\N	log-of-unexplained-locations	5
305	95	\N	about-the-scp-foundation	5
306	95	\N	object-classes	5
307	95	\N	personnel-and-character-dossier	5
308	95	\N	security-clearance-levels	5
309	95	\N	secure-facilities-locations	5
310	95	\N	task-forces	5
311	95	\N	guide-hub	5
312	95	\N	usertools	5
313	95	\N	tag-search	5
314	95	\N	meet-the-staff	5
315	95	\N	criticism-policy	5
316	95	\N	licensing-guide	5
317	95	\N	image-use-policy	5
318	95	\N	chat-guide	5
319	95	\N	deletions-guide	5
320	95	\N	seminars-hub	5
321	95	\N	donations-policy	5
322	95	\N	links	5
323	95	\N	guide-for-newbies	5
324	95	\N	how-to-write-an-scp	5
325	95	\N	faq	5
326	95	\N	site-rules	5
343	72	73	\N	1
344	72	1	\N	1
345	72	76	\N	1
346	72	69	\N	1
347	69	73	\N	1
348	69	69	\N	1
349	69	76	\N	1
350	69	\N	mywikidot-blank	1
351	76	72	\N	1
352	76	41	\N	1
353	76	42	\N	1
354	76	3	\N	1
355	76	78	\N	1
356	76	37	\N	1
357	76	77	\N	1
358	76	39	\N	1
359	76	43	\N	1
360	76	46	\N	1
361	76	47	\N	1
362	76	1	\N	1
363	76	76	\N	1
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
14	\N	Join This Wiki	system:join	1
15	\N	\N	admin:manage	1
16	\N	Page Tags List	system:page-tags-list	1
17	\N	Recent Changes	system:recent-changes	1
18	\N	Members	system:members	1
19	\N	Wiki Search	search:site	1
20	\N	\N	system:page-tags	1
21	\N	List All Pages	system:list-all-pages	1
24	\N	Search All Wikis	search:all	1
28	\N	Forum Categories	forum:start	1
29	\N	Forum Category	forum:category	1
30	\N	Forum Thread	forum:thread	1
31	\N	New Forum Thread	forum:new-thread	1
32	\N	Recent Forum Posts	forum:recent-posts	1
34	\N	Template	profile:template	1
37	\N	Congratulations, welcome to your new wiki!	start	1
38	\N	List of all wikis	system-all:all-sites	1
41	\N	Search	system-all:search	1
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
57	\N	Side	nav:side	1
58	\N	Top	nav:top	1
59	\N	Wiki Search	search:site	1
60	\N	What Is A Wiki Site	what-is-a-wiki-site	1
61	\N	Congratulations, welcome to your new wiki!	start	1
62	\N	How To Edit Pages - Quickstart	how-to-edit-pages	1
63	\N	\N	admin:manage	1
64	\N	Join This Wiki	system:join	1
65	\N	Page Tags List	system:page-tags-list	1
66	\N	Recent Changes	system:recent-changes	1
67	\N	Members	system:members	1
68	\N	\N	system:page-tags	1
69	\N	List All Pages	system:list-all-pages	1
73	\N	MyBlog	blogstart	1
75	\N	Top	nav:top	1
78	\N	Welcome to your MyWikidot Custom Installation!	start	1
79	\N	MyWikidot Info	mywikidot-info	1
80	\N	Top	nav:top	1
81	\N	How To Edit Pages - Quickstart	how-to-edit-pages	1
82	\N	Side	nav:side	1
83	\N	List wikis by tags	system-all:sites-by-tags	1
84	\N	Activity across all wikis	system-all:activity	1
85	\N	Search Users	search:users	1
86	\N	Search This Wiki	search:site	1
91	\N	Bluesoul	bluesoul	1
92	\N	Bluesoul2	bluesoul2	1
93	\N	Theme	component:theme	1
94	\N	Profile	template:profile	1
95	\N	Template	profile:template	1
96	\N	Profile	template:profile	1
97	\N	Profile	template:profile	1
98	\N	Profile	profile:bluesoul	5
99	\N	Profile	profile:bluesoul2	6
100	\N	Side	nav:side	6
101	\N	Top	nav:top	6
102	\N	Wiki Search	search:site	6
103	\N	What Is A Wiki Site	what-is-a-wiki-site	6
104	\N	Congratulations, welcome to your new wiki!	start	6
105	\N	How To Edit Pages - Quickstart	how-to-edit-pages	6
106	\N	\N	admin:manage	6
107	\N	Join This Wiki	system:join	6
108	\N	Page Tags List	system:page-tags-list	6
109	\N	Recent Changes	system:recent-changes	6
110	\N	Members	system:members	6
111	\N	\N	system:page-tags	6
112	\N	List All Pages	system:list-all-pages	6
113	\N	Profile	template:profile	6
114	\N	Profile	profile:aismallard	7
115	\N	Profile	profile:tsatpwtcotttadc	8
116	\N	Side	nav:side	1
117	\N	Top	nav:top	1
118	\N	Wiki Search	search:site	1
119	\N	What Is A Wiki Site	what-is-a-wiki-site	1
120	\N	Congratulations, welcome to your new wiki!	start	1
121	\N	How To Edit Pages - Quickstart	how-to-edit-pages	1
122	\N	\N	admin:manage	1
123	\N	Join This Wiki	system:join	1
124	\N	Page Tags List	system:page-tags-list	1
125	\N	Recent Changes	system:recent-changes	1
126	\N	Members	system:members	1
127	\N	\N	system:page-tags	1
128	\N	List All Pages	system:list-all-pages	1
129	\N	Profile	template:profile	1
130	\N	New Pages Feed	new-pages-feed	7
131	\N	Lowest Rated Pages	lowest-rated-pages	7
132	\N	_template	_template	7
133	\N	_404	_404	7
134	\N	Black Highlighter Theme	theme:black-highlighter-theme	7
135	\N	_404	theme:_404	7
136	\N	Pedantique's Proposal	pedantique-s-proposal	7
137	\N	not_a_seagull's Proposal	not-a-seagull-proposal	7
138	\N	SCP-001	scp-001	7
139	132	not_a_seagull's Proposal	not-a-seagull-proposal	7
140	132	Pedantique's Proposal	pedantique-s-proposal	7
141	\N	Random SCP	random:random-scp	7
142	\N	Random Tale	random:random-tale	7
143	\N	info:start	info:start	7
144	\N	info:end	info:end	7
145	\N	info:end	info:more	7
146	\N	info:more	info:more	7
147	\N	info:end	info:end	7
148	\N	Profile	profile:croquembouche	9
149	\N	Profile	profile:stormbreath	10
156	\N	\N	admin:manage	9
164	\N	Side	nav:side	9
165	\N	Top	nav:top	9
166	\N	Wiki Search	search:site	9
167	\N	What Is A Wiki Site	what-is-a-wiki-site	9
168	\N	Congratulations, welcome to your new wiki!	start	9
169	\N	How To Edit Pages - Quickstart	how-to-edit-pages	9
170	\N	\N	admin:manage	9
171	\N	Join This Wiki	system:join	9
172	\N	Page Tags List	system:page-tags-list	9
173	\N	Recent Changes	system:recent-changes	9
174	\N	Members	system:members	9
175	\N	\N	system:page-tags	9
176	\N	List All Pages	system:list-all-pages	9
177	\N	Profile	template:profile	9
178	\N	All Wikis	all-wikis	9
179	\N	Forum Categories	forum:start	9
180	\N	Forum Category	forum:category	9
181	\N	Forum Thread	forum:thread	9
182	\N	New Forum Thread	forum:new-thread	9
183	\N	Recent Forum Posts	forum:recent-posts	9
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
64	23	63	44	\N	t	f	f	f	f	f	0	f	4	2008-12-06 17:22:04	1	\N		f	1
65	23	64	44	\N	t	f	f	f	f	f	0	f	5	2008-12-06 17:23:00	1	\N		f	1
66	53	65	57	\N	f	f	f	f	f	t	0	f	0	2008-12-06 17:49:32	1	\N	\N	t	4
67	54	66	58	\N	f	f	f	f	f	t	0	f	0	2008-12-06 17:49:32	1	\N	\N	t	4
68	55	67	59	\N	f	f	f	f	f	t	0	f	0	2008-12-06 17:49:32	1	\N	\N	t	4
69	56	68	60	\N	f	f	f	f	f	t	0	f	0	2008-12-06 17:49:32	1	\N	\N	t	4
70	57	69	61	\N	f	f	f	f	f	t	0	f	0	2008-12-06 17:49:32	1	\N	\N	t	4
71	58	70	62	\N	f	f	f	f	f	t	0	f	0	2008-12-06 17:49:32	1	\N	\N	t	4
72	59	71	63	\N	f	f	f	f	f	t	0	f	0	2008-12-06 17:49:32	1	\N	\N	t	4
73	60	72	64	\N	f	f	f	f	f	t	0	f	0	2008-12-06 17:49:32	1	\N	\N	t	4
74	61	73	65	\N	f	f	f	f	f	t	0	f	0	2008-12-06 17:49:32	1	\N	\N	t	4
75	62	74	66	\N	f	f	f	f	f	t	0	f	0	2008-12-06 17:49:32	1	\N	\N	t	4
76	63	75	67	\N	f	f	f	f	f	t	0	f	0	2008-12-06 17:49:32	1	\N	\N	t	4
77	64	76	68	\N	f	f	f	f	f	t	0	f	0	2008-12-06 17:49:32	1	\N	\N	t	4
78	65	77	69	\N	f	f	f	f	f	t	0	f	0	2008-12-06 17:49:32	1	\N	\N	t	4
79	54	78	58	\N	t	f	f	f	f	f	0	f	1	2008-12-06 17:51:20	1	\N		f	4
80	23	79	70	\N	t	t	f	f	f	f	0	f	6	2008-12-06 19:45:29	1	\N		f	1
81	66	80	71	\N	f	f	f	f	f	t	0	f	0	2008-12-06 20:18:57	1	\N		f	1
82	66	81	71	\N	t	f	f	f	f	f	0	f	1	2008-12-06 20:30:45	1	\N		f	1
83	66	82	71	\N	t	f	f	f	f	f	0	f	2	2008-12-06 20:35:38	1	\N		f	1
84	66	83	71	\N	t	f	f	f	f	f	0	f	3	2008-12-06 20:38:15	1	\N		f	1
85	66	84	71	\N	t	f	f	f	f	f	0	f	4	2008-12-06 20:41:15	1	\N		f	1
86	67	85	72	\N	f	f	f	f	f	t	0	f	0	2008-12-06 20:41:46	1	\N		f	1
87	53	86	57	\N	t	f	f	f	f	f	0	f	1	2008-12-08 01:27:34	1	\N		f	4
88	57	87	61	\N	t	f	f	f	f	f	0	f	1	2008-12-08 01:33:53	1	\N		f	4
89	68	88	73	\N	f	f	f	f	f	t	0	f	0	2008-12-08 01:34:49	1	\N		f	4
90	68	89	73	\N	t	f	f	f	f	f	0	f	1	2008-12-08 01:35:29	1	\N		f	4
91	67	90	72	\N	t	f	f	f	f	f	0	f	1	2009-01-04 17:30:29	1	\N		f	1
92	23	91	74	\N	t	t	f	f	f	f	0	f	7	2009-01-04 17:39:22	1	\N		f	1
93	69	92	75	\N	f	f	f	f	f	t	0	f	0	2009-01-04 17:54:21	1	\N		f	1
94	32	93	33	\N	t	f	f	f	f	f	0	f	1	2009-01-04 17:57:51	1	\N		f	2
95	32	94	33	\N	t	f	f	f	f	f	0	f	2	2009-01-04 17:59:18	1	\N		f	2
96	23	95	74	\N	t	f	f	f	f	f	0	f	8	2009-01-04 18:05:19	1	\N		f	1
97	70	96	76	\N	f	f	f	f	f	t	0	f	0	2009-01-04 18:06:40	1	\N		f	1
98	70	97	76	\N	t	f	f	f	f	f	0	f	1	2009-01-04 18:08:31	1	\N		f	1
99	71	98	77	\N	f	f	f	f	f	t	0	f	0	2009-01-04 18:09:07	1	\N		f	1
100	71	99	77	\N	t	f	f	f	f	f	0	f	1	2009-01-04 19:39:54	1	\N		f	1
101	72	100	78	\N	f	f	f	f	f	t	0	f	0	2009-01-04 19:40:27	1	\N		f	1
102	73	101	79	\N	f	f	f	f	f	t	0	f	0	2009-01-04 19:46:55	1	\N		f	1
103	74	102	80	\N	f	f	f	f	f	t	0	f	0	2009-01-04 19:49:22	1	\N		f	2
104	75	103	81	\N	f	f	f	f	f	t	0	f	0	2009-01-04 19:50:30	1	\N		f	2
105	76	104	82	\N	f	f	f	f	f	t	0	f	0	2009-01-04 19:51:32	1	\N		f	1
106	77	105	83	\N	f	f	f	f	f	t	0	f	0	2009-01-04 19:52:44	1	\N		f	1
107	40	106	43	\N	t	f	f	f	f	f	0	f	2	2009-01-04 19:54:02	1	\N		f	1
108	78	107	84	\N	f	f	f	f	f	t	0	f	0	2009-01-04 19:54:26	1	\N		f	1
109	79	108	85	\N	f	f	f	f	f	t	0	f	0	2009-01-04 19:55:43	1	\N		f	1
110	80	109	86	\N	f	f	f	f	f	t	0	f	0	2009-01-04 19:56:53	1	\N		f	1
115	85	114	91	\N	f	f	f	f	f	t	0	f	0	2020-06-02 02:37:49	1	\N		f	1
116	85	117	91	\N	t	f	f	f	f	f	0	f	1	2020-06-02 02:42:45	1	\N	new revision	f	1
117	86	118	92	\N	f	f	f	f	f	t	0	f	0	2020-06-02 14:46:19	1	\N		f	1
118	85	119	91	\N	t	f	f	f	f	f	0	f	2	2020-06-02 16:21:19	1	\N		f	1
119	85	120	91	\N	t	f	f	f	f	f	0	f	3	2020-06-02 16:29:39	1	\N		f	1
120	85	121	91	\N	t	f	f	f	f	f	0	f	4	2020-06-02 16:50:34	1	\N		f	1
121	85	122	91	\N	t	f	f	f	f	f	0	f	5	2020-06-02 16:51:18	1	\N		f	1
122	85	123	91	\N	t	f	f	f	f	f	0	f	6	2020-06-02 16:57:43	1	\N		f	1
123	85	124	91	\N	t	f	f	f	f	f	0	f	7	2020-06-02 16:58:43	1	\N		f	1
124	85	125	91	\N	t	f	f	f	f	f	0	f	8	2020-06-02 17:00:33	1	\N		f	1
125	85	126	91	\N	t	f	f	f	f	f	0	f	9	2020-06-02 17:01:27	1	\N		f	1
127	85	128	91	\N	t	f	f	f	f	f	0	f	10	2020-06-02 17:05:52	1	\N		f	1
128	85	129	91	\N	t	f	f	f	f	f	0	f	11	2020-06-02 17:06:49	1	\N		f	1
129	85	130	91	\N	t	f	f	f	f	f	0	f	12	2020-06-02 17:16:19	1	\N		f	1
130	85	131	91	\N	t	f	f	f	f	f	0	f	13	2020-06-02 17:20:18	1	\N		f	1
131	85	132	91	\N	t	f	f	f	f	f	0	f	14	2020-06-02 17:22:42	1	\N		f	1
132	85	133	91	\N	t	f	f	f	f	f	0	f	15	2020-06-02 17:25:48	1	\N		f	1
135	85	136	91	\N	t	f	f	f	f	f	0	f	16	2020-06-02 17:30:56	1	\N		f	1
136	85	137	91	\N	t	f	f	f	f	f	0	f	17	2020-06-02 17:35:05	1	\N		f	1
137	85	138	91	\N	t	f	f	f	f	f	0	f	18	2020-06-02 17:39:26	1	\N		f	1
138	85	139	91	\N	t	f	f	f	f	f	0	f	19	2020-06-02 17:43:04	1	\N		f	1
139	85	140	91	\N	t	f	f	f	f	f	0	f	20	2020-06-02 17:50:34	1	\N		f	1
140	85	141	91	\N	t	f	f	f	f	f	0	f	21	2020-06-02 19:07:45	1	\N		f	1
141	85	142	91	\N	t	f	f	f	f	f	0	f	22	2020-06-02 19:11:20	1	\N		f	1
142	85	143	91	\N	t	f	f	f	f	f	0	f	23	2020-06-02 19:14:31	1	\N		f	1
143	85	144	91	\N	t	f	f	f	f	f	0	f	24	2020-06-02 19:24:21	1	\N		f	1
144	85	145	91	\N	t	f	f	f	f	f	0	f	25	2020-06-02 20:05:59	1	\N		f	1
145	87	146	93	\N	f	f	f	f	f	t	0	f	0	2020-06-02 20:46:38	1	\N	Let's try this.	f	1
146	88	147	94	\N	f	f	f	f	f	t	0	f	0	2020-06-02 21:35:30	1	\N		f	2
147	89	148	95	\N	f	f	f	f	f	t	0	f	0	2020-06-02 21:38:10	1	\N		f	1
148	90	149	96	\N	f	f	f	f	f	t	0	f	0	2020-06-02 21:38:36	1	\N		f	1
149	91	150	97	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:02:25	1	\N		f	3
150	92	151	98	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:03:20	5	\N	\N	t	3
151	93	152	99	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:11:30	6	\N	\N	t	3
152	94	153	100	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:15:23	6	\N	\N	t	5
153	95	154	101	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:15:24	6	\N	\N	t	5
154	96	155	102	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:15:24	6	\N	\N	t	5
155	97	156	103	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:15:24	6	\N	\N	t	5
156	98	157	104	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:15:24	6	\N	\N	t	5
157	99	158	105	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:15:24	6	\N	\N	t	5
158	100	159	106	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:15:24	6	\N	\N	t	5
159	101	160	107	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:15:24	6	\N	\N	t	5
160	102	161	108	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:15:24	6	\N	\N	t	5
161	103	162	109	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:15:24	6	\N	\N	t	5
162	104	163	110	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:15:24	6	\N	\N	t	5
163	105	164	111	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:15:24	6	\N	\N	t	5
164	106	165	112	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:15:24	6	\N	\N	t	5
165	107	166	113	\N	f	f	f	f	f	t	0	f	0	2020-06-02 22:15:24	6	\N	\N	t	5
166	76	167	82	\N	t	f	f	f	f	f	0	f	1	2020-06-02 22:44:59	1	\N		f	1
167	76	167	82	\N	f	f	t	f	f	f	0	f	2	2020-06-02 22:45:50	1	\N	Uploaded file "badwikidot.png".	f	1
168	76	167	82	\N	f	f	t	f	f	f	0	f	3	2020-06-02 22:47:47	1	\N	Uploaded file "black.png".	f	1
169	76	167	82	\N	f	f	t	f	f	f	0	f	4	2020-06-02 22:47:56	1	\N	Uploaded file "default.png".	f	1
170	76	167	82	\N	f	f	t	f	f	f	0	f	5	2020-06-02 22:48:10	1	\N	Uploaded file "main.png".	f	1
171	76	167	82	\N	f	f	t	f	f	f	0	f	6	2020-06-02 22:48:42	1	\N	Uploaded file "Twitter-icon.png".	f	1
172	76	167	82	\N	f	f	t	f	f	f	0	f	7	2020-06-02 22:48:50	1	\N	Uploaded file "Tumblr-icon.png".	f	1
173	76	167	82	\N	f	f	t	f	f	f	0	f	8	2020-06-02 22:48:57	1	\N	Uploaded file "tumblr2.png".	f	1
174	76	167	82	\N	f	f	t	f	f	f	0	f	9	2020-06-02 22:49:05	1	\N	Uploaded file "series.png".	f	1
175	76	167	82	\N	f	f	t	f	f	f	0	f	10	2020-06-02 22:49:14	1	\N	Uploaded file "Reddit-icon.png".	f	1
176	76	167	82	\N	f	f	t	f	f	f	0	f	11	2020-06-02 22:49:30	1	\N	Uploaded file "icon-Twitter-2020.png".	f	1
177	76	167	82	\N	f	f	t	f	f	f	0	f	12	2020-06-02 22:49:38	1	\N	Uploaded file "icon-Tumblr-2020.png".	f	1
178	76	167	82	\N	f	f	t	f	f	f	0	f	13	2020-06-02 22:49:48	1	\N	Uploaded file "icon-Instagram-2020.png".	f	1
179	76	167	82	\N	f	f	t	f	f	f	0	f	14	2020-06-02 22:49:56	1	\N	Uploaded file "icon-Reddit-2020.png".	f	1
180	76	167	82	\N	f	f	t	f	f	f	0	f	15	2020-06-02 22:50:09	1	\N	Uploaded file "icon-Facebook-2020.png".	f	1
181	76	167	82	\N	f	f	t	f	f	f	0	f	16	2020-06-02 22:50:17	1	\N	Uploaded file "icon-DeviantArt-2020.png".	f	1
182	76	167	82	\N	f	f	t	f	f	f	0	f	17	2020-06-02 22:50:25	1	\N	Uploaded file "home.png".	f	1
183	76	167	82	\N	f	f	t	f	f	f	0	f	18	2020-06-02 22:50:34	1	\N	Uploaded file "help.png".	f	1
184	76	167	82	\N	f	f	t	f	f	f	0	f	19	2020-06-02 22:50:43	1	\N	Uploaded file "FB-icon.png".	f	1
185	76	167	82	\N	f	f	t	f	f	f	0	f	20	2020-06-02 22:50:51	1	\N	Uploaded file "expand.png".	f	1
186	76	167	82	\N	f	f	t	f	f	f	0	f	21	2020-06-02 22:51:07	1	\N	Uploaded file "DA-icon.png".	f	1
187	69	168	75	\N	t	f	f	f	f	f	0	f	1	2020-06-02 22:52:12	1	\N		f	1
188	69	169	75	\N	t	f	f	f	f	f	0	f	2	2020-06-02 22:52:36	1	\N		f	1
189	69	170	75	\N	t	f	f	f	f	f	0	f	3	2020-06-02 22:53:19	1	\N		f	1
190	94	171	100	\N	t	f	f	f	f	f	0	f	1	2020-06-02 22:55:35	1	\N		f	5
191	94	171	100	\N	f	f	t	f	f	f	0	f	2	2020-06-02 22:55:46	1	\N	Uploaded file "Twitter-icon.png".	f	5
192	94	171	100	\N	f	f	t	f	f	f	0	f	3	2020-06-02 22:55:53	1	\N	Uploaded file "Tumblr-icon.png".	f	5
193	94	171	100	\N	f	f	t	f	f	f	0	f	4	2020-06-02 22:56:00	1	\N	Uploaded file "tumblr2.png".	f	5
194	94	171	100	\N	f	f	t	f	f	f	0	f	5	2020-06-02 22:56:07	1	\N	Uploaded file "series.png".	f	5
195	94	171	100	\N	f	f	t	f	f	f	0	f	6	2020-06-02 22:56:19	1	\N	Uploaded file "Reddit-icon.png".	f	5
196	94	171	100	\N	f	f	t	f	f	f	0	f	7	2020-06-02 22:56:34	1	\N	Uploaded file "paypal.jpg".	f	5
197	94	171	100	\N	f	f	t	f	f	f	0	f	8	2020-06-02 22:56:42	1	\N	Uploaded file "main.png".	f	5
198	94	171	100	\N	f	f	t	f	f	f	0	f	9	2020-06-02 22:56:50	1	\N	Uploaded file "icon-Twitter-2020.png".	f	5
199	94	171	100	\N	f	f	t	f	f	f	0	f	10	2020-06-02 22:57:04	1	\N	Uploaded file "icon-Tumblr-2020.png".	f	5
200	94	171	100	\N	f	f	t	f	f	f	0	f	11	2020-06-02 22:57:11	1	\N	Uploaded file "icon-Reddit-2020.png".	f	5
201	94	171	100	\N	f	f	t	f	f	f	0	f	12	2020-06-02 22:57:18	1	\N	Uploaded file "icon-Instagram-2020.png".	f	5
202	94	171	100	\N	f	f	t	f	f	f	0	f	13	2020-06-02 22:57:26	1	\N	Uploaded file "icon-Facebook-2020.png".	f	5
203	94	171	100	\N	f	f	t	f	f	f	0	f	14	2020-06-02 22:57:38	1	\N	Uploaded file "icon-DeviantArt-2020.png".	f	5
204	94	171	100	\N	f	f	t	f	f	f	0	f	15	2020-06-02 22:57:48	1	\N	Uploaded file "home.png".	f	5
205	94	171	100	\N	f	f	t	f	f	f	0	f	16	2020-06-02 22:57:57	1	\N	Uploaded file "help.png".	f	5
206	94	171	100	\N	f	f	t	f	f	f	0	f	17	2020-06-02 22:58:05	1	\N	Uploaded file "forum.png".	f	5
207	94	171	100	\N	f	f	t	f	f	f	0	f	18	2020-06-02 22:58:15	1	\N	Uploaded file "FB-icon.png".	f	5
208	94	171	100	\N	f	f	t	f	f	f	0	f	19	2020-06-02 22:58:23	1	\N	Uploaded file "expand.png".	f	5
209	94	171	100	\N	f	f	t	f	f	f	0	f	20	2020-06-02 22:58:34	1	\N	Uploaded file "default.png".	f	5
210	94	171	100	\N	f	f	t	f	f	f	0	f	21	2020-06-02 22:58:44	1	\N	Uploaded file "DA-icon.png".	f	5
211	94	171	100	\N	f	f	t	f	f	f	0	f	22	2020-06-02 22:58:51	1	\N	Uploaded file "blank.png".	f	5
212	94	171	100	\N	f	f	t	f	f	f	0	f	23	2020-06-02 22:59:00	1	\N	Uploaded file "black.png".	f	5
213	95	172	101	\N	t	f	f	f	f	f	0	f	1	2020-06-02 22:59:32	1	\N		f	5
218	72	177	78	\N	t	f	f	f	f	f	0	f	1	2020-06-02 23:04:03	1	\N		f	1
219	76	167	82	\N	f	f	t	f	f	f	0	f	22	2020-06-02 23:09:26	1	\N	Uploaded file "forum.png".	f	1
220	108	178	114	\N	f	f	f	f	f	t	0	f	0	2020-06-03 05:34:53	7	\N	\N	t	3
221	109	179	115	\N	f	f	f	f	f	t	0	f	0	2020-06-03 06:22:03	8	\N	\N	t	3
222	72	180	78	\N	t	f	f	f	f	f	0	f	2	2020-06-06 18:51:37	1	\N	Reverted to page revision number 0	f	1
223	69	181	75	\N	t	f	f	f	f	f	0	f	4	2020-06-06 18:51:56	1	\N	Reverted to page revision number 0	f	1
224	76	182	82	\N	t	f	f	f	f	f	0	f	23	2020-06-06 18:52:18	1	\N	Reverted to page revision number 0	f	1
225	110	183	116	\N	f	f	f	f	f	t	0	f	0	2020-06-06 18:59:55	1	\N	\N	t	6
226	111	184	117	\N	f	f	f	f	f	t	0	f	0	2020-06-06 18:59:55	1	\N	\N	t	6
227	112	185	118	\N	f	f	f	f	f	t	0	f	0	2020-06-06 18:59:55	1	\N	\N	t	6
228	113	186	119	\N	f	f	f	f	f	t	0	f	0	2020-06-06 18:59:55	1	\N	\N	t	6
229	114	187	120	\N	f	f	f	f	f	t	0	f	0	2020-06-06 18:59:55	1	\N	\N	t	6
230	115	188	121	\N	f	f	f	f	f	t	0	f	0	2020-06-06 18:59:55	1	\N	\N	t	6
231	116	189	122	\N	f	f	f	f	f	t	0	f	0	2020-06-06 18:59:55	1	\N	\N	t	6
232	117	190	123	\N	f	f	f	f	f	t	0	f	0	2020-06-06 18:59:55	1	\N	\N	t	6
233	118	191	124	\N	f	f	f	f	f	t	0	f	0	2020-06-06 18:59:55	1	\N	\N	t	6
234	119	192	125	\N	f	f	f	f	f	t	0	f	0	2020-06-06 18:59:55	1	\N	\N	t	6
235	120	193	126	\N	f	f	f	f	f	t	0	f	0	2020-06-06 18:59:55	1	\N	\N	t	6
236	121	194	127	\N	f	f	f	f	f	t	0	f	0	2020-06-06 18:59:55	1	\N	\N	t	6
237	122	195	128	\N	f	f	f	f	f	t	0	f	0	2020-06-06 18:59:55	1	\N	\N	t	6
238	123	196	129	\N	f	f	f	f	f	t	0	f	0	2020-06-06 18:59:55	1	\N	\N	t	6
239	72	197	78	\N	t	f	f	f	f	f	0	f	3	2020-06-06 23:40:42	1	\N		f	1
240	124	198	130	\N	f	f	f	f	f	t	0	f	0	2020-06-07 23:32:02	7	\N		f	1
241	125	199	131	\N	f	f	f	f	f	t	0	f	0	2020-06-08 02:01:07	7	\N		f	1
242	126	200	132	\N	f	f	f	f	f	t	0	f	0	2020-06-08 02:10:26	7	\N		f	1
243	127	201	133	\N	f	f	f	f	f	t	0	f	0	2020-06-08 02:15:43	7	\N	coldpost time	f	1
244	127	202	133	\N	t	f	f	f	f	f	0	f	1	2020-06-08 02:15:55	7	\N		f	1
245	128	203	134	\N	f	f	f	f	f	t	0	f	0	2020-06-08 02:20:33	7	\N		f	1
246	129	204	135	\N	f	f	f	f	f	t	0	f	0	2020-06-08 02:24:08	7	\N		f	1
247	130	205	136	\N	f	f	f	f	f	t	0	f	0	2020-06-08 02:25:22	7	\N		f	1
248	131	206	137	\N	f	f	f	f	f	t	0	f	0	2020-06-08 02:26:02	7	\N		f	1
249	132	207	138	\N	f	f	f	f	f	t	0	f	0	2020-06-08 02:37:38	7	\N		f	1
250	130	208	136	\N	t	f	f	f	f	f	0	f	1	2020-06-08 02:38:05	7	\N		f	1
251	131	206	139	\N	f	f	f	f	t	f	0	f	1	2020-06-08 02:38:13	7	\N	Parent page set to: "scp-001".	f	1
252	130	208	140	\N	f	f	f	f	t	f	0	f	2	2020-06-08 02:38:20	7	\N	Parent page set to: "scp-001".	f	1
253	133	209	141	\N	f	f	f	f	f	t	0	f	0	2020-06-08 02:51:34	7	\N		f	1
254	134	210	142	\N	f	f	f	f	f	t	0	f	0	2020-06-08 02:51:56	7	\N		f	1
255	135	211	143	\N	f	f	f	f	f	t	0	f	0	2020-06-08 02:53:31	7	\N		f	1
256	136	212	144	\N	f	f	f	f	f	t	0	f	0	2020-06-08 02:53:57	7	\N		f	1
257	136	212	145	\N	f	f	f	t	f	f	0	f	1	2020-06-08 02:54:32	7	\N	Page name changed: "info:end" to "info:more".	f	1
258	136	212	146	\N	f	t	f	f	f	f	0	f	2	2020-06-08 02:54:40	7	\N		f	1
259	137	213	147	\N	f	f	f	f	f	t	0	f	0	2020-06-08 02:54:59	7	\N		f	1
260	138	214	148	\N	f	f	f	f	f	t	0	f	0	2020-06-08 03:04:20	9	\N	\N	t	3
261	139	215	149	\N	f	f	f	f	f	t	0	f	0	2020-06-08 03:06:02	10	\N	\N	t	3
262	140	216	150	\N	f	f	f	f	f	t	0	f	0	2020-06-08 03:06:18	9	\N	\N	t	7
263	141	217	151	\N	f	f	f	f	f	t	0	f	0	2020-06-08 03:06:18	9	\N	\N	t	7
264	142	218	152	\N	f	f	f	f	f	t	0	f	0	2020-06-08 03:06:18	9	\N	\N	t	7
265	143	219	153	\N	f	f	f	f	f	t	0	f	0	2020-06-08 03:06:18	9	\N	\N	t	7
266	144	220	154	\N	f	f	f	f	f	t	0	f	0	2020-06-08 03:06:18	9	\N	\N	t	7
267	145	221	155	\N	f	f	f	f	f	t	0	f	0	2020-06-08 03:06:18	9	\N	\N	t	7
268	146	222	156	\N	f	f	f	f	f	t	0	f	0	2020-06-08 03:06:18	9	\N	\N	t	7
269	147	223	157	\N	f	f	f	f	f	t	0	f	0	2020-06-08 03:06:18	9	\N	\N	t	7
270	148	224	158	\N	f	f	f	f	f	t	0	f	0	2020-06-08 03:06:18	9	\N	\N	t	7
271	149	225	159	\N	f	f	f	f	f	t	0	f	0	2020-06-08 03:06:18	9	\N	\N	t	7
272	150	226	160	\N	f	f	f	f	f	t	0	f	0	2020-06-08 03:06:18	9	\N	\N	t	7
273	151	227	161	\N	f	f	f	f	f	t	0	f	0	2020-06-08 03:06:18	9	\N	\N	t	7
274	152	228	162	\N	f	f	f	f	f	t	0	f	0	2020-06-08 03:06:18	9	\N	\N	t	7
275	153	229	163	\N	f	f	f	f	f	t	0	f	0	2020-06-08 03:06:18	9	\N	\N	t	7
276	72	230	78	\N	t	f	f	f	f	f	0	f	4	2020-06-09 00:56:39	1	\N		f	1
283	72	237	78	\N	t	f	f	f	f	f	0	f	5	2020-06-09 01:09:35	1	\N		f	1
284	72	238	78	\N	t	f	f	f	f	f	0	f	6	2020-06-09 01:30:40	1	\N	Reverted to page revision number 3	f	1
285	72	239	78	\N	t	f	f	f	f	f	0	f	7	2020-06-09 01:31:16	1	\N	Reverted to page revision number 0	f	1
286	72	240	78	\N	t	f	f	f	f	f	0	f	8	2020-06-09 01:33:45	1	\N		f	1
287	72	241	78	\N	t	f	f	f	f	f	0	f	9	2020-06-09 01:45:27	1	\N		f	1
288	72	242	78	\N	t	f	f	f	f	f	0	f	10	2020-06-09 02:22:09	1	\N		f	1
289	131	243	139	\N	t	f	f	f	f	f	0	f	2	2020-06-10 22:18:12	1	\N		f	1
290	131	244	139	\N	t	f	f	f	f	f	0	f	3	2020-06-10 22:19:40	1	\N	Reverted to page revision number 0	f	1
297	85	251	91	\N	t	f	f	f	f	f	0	f	26	2020-06-10 22:28:19	1	\N		f	1
298	85	252	91	\N	t	f	f	f	f	f	0	f	27	2020-06-10 22:28:48	1	\N		f	1
299	131	253	139	\N	t	f	f	f	f	f	0	f	4	2020-06-10 22:29:21	1	\N		f	1
300	131	254	139	\N	t	f	f	f	f	f	0	f	5	2020-06-10 22:54:05	1	\N		f	1
301	131	255	139	\N	t	f	f	f	f	f	0	f	6	2020-06-10 22:59:22	1	\N	Reverted to page revision number 4	f	1
302	131	256	139	\N	t	f	f	f	f	f	0	f	7	2020-06-10 23:04:13	1	\N		f	1
303	131	257	139	\N	t	f	f	f	f	f	0	f	8	2020-06-10 23:04:46	1	\N		f	1
304	131	258	139	\N	t	f	f	f	f	f	0	f	9	2020-06-10 23:05:17	1	\N		f	1
305	131	259	139	\N	t	f	f	f	f	f	0	f	10	2020-06-10 23:06:25	1	\N		f	1
306	131	260	139	\N	t	f	f	f	f	f	0	f	11	2020-06-10 23:11:52	1	\N		f	1
307	131	261	139	\N	t	f	f	f	f	f	0	f	12	2020-06-10 23:37:51	1	\N		f	1
308	131	262	139	\N	t	f	f	f	f	f	0	f	13	2020-06-10 23:48:01	1	\N		f	1
309	131	263	139	\N	t	f	f	f	f	f	0	f	14	2020-06-11 00:07:45	1	\N		f	1
310	131	264	139	\N	t	f	f	f	f	f	0	f	15	2020-06-11 00:11:54	1	\N		f	1
311	154	265	164	\N	f	f	f	f	f	t	0	f	0	2020-06-11 16:17:16	9	\N	\N	t	8
312	155	266	165	\N	f	f	f	f	f	t	0	f	0	2020-06-11 16:17:16	9	\N	\N	t	8
313	156	267	166	\N	f	f	f	f	f	t	0	f	0	2020-06-11 16:17:16	9	\N	\N	t	8
314	157	268	167	\N	f	f	f	f	f	t	0	f	0	2020-06-11 16:17:16	9	\N	\N	t	8
315	158	269	168	\N	f	f	f	f	f	t	0	f	0	2020-06-11 16:17:16	9	\N	\N	t	8
316	159	270	169	\N	f	f	f	f	f	t	0	f	0	2020-06-11 16:17:16	9	\N	\N	t	8
317	160	271	170	\N	f	f	f	f	f	t	0	f	0	2020-06-11 16:17:16	9	\N	\N	t	8
318	161	272	171	\N	f	f	f	f	f	t	0	f	0	2020-06-11 16:17:16	9	\N	\N	t	8
319	162	273	172	\N	f	f	f	f	f	t	0	f	0	2020-06-11 16:17:16	9	\N	\N	t	8
320	163	274	173	\N	f	f	f	f	f	t	0	f	0	2020-06-11 16:17:16	9	\N	\N	t	8
321	164	275	174	\N	f	f	f	f	f	t	0	f	0	2020-06-11 16:17:16	9	\N	\N	t	8
322	165	276	175	\N	f	f	f	f	f	t	0	f	0	2020-06-11 16:17:16	9	\N	\N	t	8
323	166	277	176	\N	f	f	f	f	f	t	0	f	0	2020-06-11 16:17:16	9	\N	\N	t	8
324	167	278	177	\N	f	f	f	f	f	t	0	f	0	2020-06-11 16:17:16	9	\N	\N	t	8
325	158	279	168	\N	t	f	f	f	f	f	0	f	1	2020-06-11 16:17:40	9	\N		f	8
326	168	280	178	\N	f	f	f	f	f	t	0	f	0	2020-06-11 16:18:12	9	\N		f	8
327	168	281	178	\N	t	f	f	f	f	f	0	f	1	2020-06-11 16:18:22	9	\N		f	8
328	139	282	149	\N	t	f	f	f	f	f	0	f	1	2020-06-11 20:23:07	10	\N		f	3
329	158	283	168	\N	t	f	f	f	f	f	0	f	2	2020-06-12 05:43:55	9	\N		f	8
330	158	284	168	\N	t	f	f	f	f	f	0	f	3	2020-06-12 05:57:41	9	\N		f	8
331	158	285	168	\N	t	f	f	f	f	f	0	f	4	2020-06-12 06:02:04	9	\N		f	8
332	158	286	168	\N	t	f	f	f	f	f	0	f	5	2020-06-12 07:28:16	9	\N		f	8
333	169	287	179	\N	f	f	f	f	f	t	0	f	0	2020-06-12 07:28:32	9	\N	\N	t	8
334	170	288	180	\N	f	f	f	f	f	t	0	f	0	2020-06-12 07:28:32	9	\N	\N	t	8
335	171	289	181	\N	f	f	f	f	f	t	0	f	0	2020-06-12 07:28:32	9	\N	\N	t	8
336	172	290	182	\N	f	f	f	f	f	t	0	f	0	2020-06-12 07:28:32	9	\N	\N	t	8
337	173	291	183	\N	f	f	f	f	f	t	0	f	0	2020-06-12 07:28:32	9	\N	\N	t	8
338	158	292	168	\N	t	f	f	f	f	f	0	f	6	2020-06-12 08:24:05	9	\N		f	8
339	158	293	168	\N	t	f	f	f	f	f	0	f	7	2020-06-12 09:11:29	9	\N		f	8
340	158	294	168	\N	t	f	f	f	f	f	0	f	8	2020-06-12 10:36:37	9	\N		f	8
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
100	[!--\nWelcome to your MyWikidot Custom Installation!\n--]\nCongratulations, you have successfully configured and launched your Wikidot custom installation!\n+ What to do next\n++ Experienced Wikidot users should [[[mywikidot-info|start here]]].\n++ Customize this wiki\nWikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n++ Customize the default templates\nThere are 2 initial default templates for other wikis. One is located at [[[template-en::start|template-en]]] and the other at [[[template-blog::start|template-blog]]]. If someone creates a new wiki, these are presented as choices and the selected template is cloned to the new wiki's address. You should customize [[[template-en::start|template-en]]] and [[[template-blog::start|template-blog]]] to suit your needs.\n++ Create more templates\nSimply create new wikis with **web site names** starting with "template-" (e.g. "template-pl", "template-recipes") and your users will be have even more choices for the basic wiki layout they want to start with. \n++ Visit Wikidot.org\nGo to **[*http://www.wikidot.org www.wikidot.org]** -- home of the Wikidot open source software -- for extra documentation, howtos, tips and support.\n++ Visit the Wikidot Community Site\nGo to **[*http://community.wikidot.com community.wikidot.com]** -- for even more tips, tricks and help from a very active community of Wikidot users.\n++ Visit the MyWikidot.local Project Site\nGo to **[*http://my-wd-local.wikidot.com/ my-wd-local.wikidot.com]** -- for tips, discussions and how-to articles.\n---------------\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org] and the developers discussion at [*http://groups.google.com/group/wikidot Wikidot dev-list].\n+ Search all wikis\n[[module SearchAll]]\n+ Search users\n[[module SearchUsers]]
15	[[note]]\nPlease change this page according to your policy (configure first using [[[admin:manage|Site Manager]]]) and remove this note.\n[[/note]]\n\n+ Who can join?\n\nYou can write here who can become a member of this site.\n\n+ Join!\n\nSo you want to become a member of this site? Tell us why and apply now!\n\n[[module MembershipApply]] \n\nOr, if you already know a "secret password", go for it!\n\n[[module MembershipByPassword]]
16	[[module ManageSite]]
17	[[module TagCloud limit="200" target="system:page-tags"]]\n\n[!--\n\nYou can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module \nBut if you want to keep the tag functionality working - do not remove these modules.\n\n--]
18	[[module SiteChanges]]
19	+ Members:\n\n[[module Members]]\n\n+ Moderators\n\n[[module Members group="moderators"]]\n\n+ Admins\n\n[[module Members group="admins"]]
20	[[module Search]]\n\n[!-- please do not remove or change this page if you want to keep the search function working --]
21	[[div style="float:right; width: 50%;"]]\n[[module TagCloud limit="200" target="system:page-tags"]]\n[[/div]]\n[[module PagesByTag]]\n\n[!--\n\nYou can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module \nBut if you want to keep the tag functionality working - do not remove these modules.\n\n--]
22	[[module Pages preview="true"]]
70	If you are allowed to edit pages in this Site, simply click on //edit// button at the bottom of the page. This will open an editor with a toolbar pallette with options.\n\nTo create a link to a new page, use syntax: {{``[[[new page name]]]``}} or {{``[[[new page name | text to display]]]``}}. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!\n\nAlthough creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit [*http://www.wikidot.org/doc Documentation pages] (at wikidot.org) to learn more.
71	[[module ManageSite]]
25	[[module SearchAll]]
72	[[note]]\nPlease change this page according to your policy (configure first using [[[admin:manage|Site Manager]]]) and remove this note.\n[[/note]]\n\n+ Who can join?\n\nYou can write here who can become a member of this site.\n\n+ Join!\n\nSo you want to become a member of this site? Tell us why and apply now!\n\n[[module MembershipApply]] \n\nOr, if you already know a "secret password", go for it!\n\n[[module MembershipByPassword]]
73	[[module TagCloud limit="200" target="system:page-tags"]]\n\n[!--\n\nYou can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module \nBut if you want to keep the tag functionality working - do not remove these modules.\n\n--]
30	[[module ForumStart]]\n[!-- please do not alter this page if you want to keep your forum working --]
31	[[module ForumCategory]]\n\n[!-- please do not alter this page if you want to keep your forum working --]
32	[[module ForumThread]]\n\n[!-- please do not alter this page if you want to keep your forum working --]
33	[[module ForumNewThread]]\n\n[!-- please do not alter this page if you want to keep your forum working --]
34	[[module RecentPosts]]\n\n[!-- please do not alter this page if you want to keep your forum working --]
36	Profile has not been created (yet).
39	++ If this is your first site\n\nThen there are some things you need to know:\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org].
101	[!--\nMyWikidot Info\n--]\n+++ January 6, 2008\nThis installation was created using [*http://www.remastersys.klikit-linux.com/ Remastersys Backup]\n\nAs distributed, this is essentially "**Wikidot-In-A-Box**" - a fully functioning and configured installation of **Wikidot v1 Open Source rev 393**. You can use the normal "//Get A New Wiki//" process to create the following sites without any other configuration needed.\n++ Sites\n* mytest\n* myblog\n* mysandbox\n\nYou can create new templates and sites by using the normal "//Get A New Wiki//" process, but you must then edit the //**/etc/hosts**// file so you can access the new sites. The easiest way to do this is using the **Nautilus File Browser** with //root// user access.\n++ How to Edit The HOSTS File\n* Open a terminal window and type:\n* sudo nautilus\n* Enter your password.\nThe Nautilus File Browser opens with root user access to files.\n* Navigate to the **/etc** folder and find the **hosts** file.\n* Right-click and **Open with "Text Editor"**\n* Edit the file so it resembles below, adding the names of your new templates or sites (this is your hosts file as it is distributed).\n> 127.0.0.1 localhost\n> 127.0.1.1 mywikidot\n> @@# required for base install@@\n> ##red|127.0.0.1 www.mywikidot.com##\n> ##red|127.0.0.1 profiles.mywikidot.com##\n> ##red|127.0.0.1 wikifiles.mywikidot.com##\n> ##red|127.0.0.1 template-en.mywikidot.com##\n> @@|# add new template sites if desired@@\n> ##red|127.0.0.1 template-blog.mywikidot.com##\n> @@# add new site names here@@\n> ##red|127.0.0.1 mytest.mywikidot.com##\n> ##red|127.0.0.1 myblog.mywikidot.com##\n> ##red|127.0.0.1 mysandbox.mywikidot.com##\n>  \n> @@# The following lines are desirable for IPv6 capable hosts@@\n> ::1     ip6-localhost ip6-loopback\n> fe00::0 ip6-localnet\n> ff00::0 ip6-mcastprefix\n> ff02::1 ip6-allnodes\n> ff02::2 ip6-allrouters\n> ff02::3 ip6-allhosts\n* Save the file.\nAs distributed, this hosts file will allow you to create and access 3 new sites: **mytest.mywikidot.com**, **myblog.mywikidot.com** and **mysandbox.mywikidot.com** without the need to edit the hosts file again. If you create new templates or sites, just edit the hosts file and add your new template(s) and site(s) to the list and you will be able to access them. You don't even have to restart any services or reopen your browser!\n++ Other Resources\nI have a Wikidot site dedicated to the installation, configuration and general tweaking of the open source version of Wikidot. Please visit [[size 125%]][*http://my-wd-local.wikidot.com/ MyWikidot.local][[/size]] for current information and more tips for using Wikidot on your own hardware or virtual machine.\n\nI have enjoyed putting this custom installation together and hope you enjoy using it!\n-Ed Johnson
41	Below is the list of public visible Wikis hosted at this service:\n\n[[module ListAllWikis]]
102	* [# Sample Menu]\n * [[[www::start|MyWikidot Home]]]\n * [[[www::mywikidot-info|Experienced users]]]\n* [# Edit/Print]\n * [[button edit text="Edit This Page"]]\n * [[button print text="Print This Page"]]\n* [# Admin]\n * [[[nav:top|Edit Top Navigation]]]\n * [[[nav:side|Edit Side Navigation]]]\n * [[[admin:manage|Site Manager]]]\n\n[!-- top nav menu, use only one bulleted list above --]
103	[!--\nHow To Edit Pages - Quickstart\n--]\nIf you are allowed to edit pages in this Site, simply click on //edit// button at the bottom of the page. This will open an editor with a toolbar pallette with options.\n\nTo create a link to a new page, use syntax: {{``[[[new page name]]]``}} or {{``[[[new page name | text to display]]]``}}. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!\n\nAlthough creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit [*http://www.wikidot.org/doc Documentation pages] (at wikidot.org) to learn more.
104	* [[[start | Welcome page]]]\n\n* [[[What is a Wiki?]]]\n* [[[How to edit pages?]]]\n* [[[new-site | Get a new wiki!]]]\n\n+ All wikis\n\n* [[[system-all:activity | Recent activity]]]\n* [[[system-all:all-sites | All wikis]]]\n* [[[system-all:sites-by-tags | Wikis by tags]]]\n* [[[system-all:search | Search]]]\n\n+ This wiki\n\n* [[[system: join | How to join this site?]]]\n* [[[system:members | Site members]]]\n\n* [[[system: Recent changes]]]\n* [[[system: List all pages]]]\n* [[[system:page-tags-list|Page Tags]]]\n\n* [[[admin:manage|Site Manager]]]\n\n++ Page tags\n[[module TagCloud minFontSize="80%" maxFontSize="200%"  maxColor="8,8,64" minColor="100,100,128" target="system:page-tags" limit="30"]]\n\n++ Add a new page\n[[module NewPage size="15" button="new page"]]\n\n= [[size 80%]][[[nav:side | edit this panel]]][[/size]]
45	[[=]]\n+ Search all Wikis\n\nPerform a search through all public and visible wikis.\n\n[[module SearchAll]]\n\n---------------\n\n+ Search users\n\nTo look for someone, please enter:\n\n* email address of a person you are looking for (this will look for exact match)\n* any part of the screen name or realname (lists all Users matching the query)\n\n[[module SearchUsers]]\n\n[[/=]]
105	[!--\nList wikis by tags\n--]\n[[module SitesTagCloud limit="100" target="system-all:sites-by-tags"]]\n\n\n[[module SitesListByTag]]
107	[!--\nActivity across all wikis\n--]\n[[table]]\n[[row]]\n[[cell style="width: 45%; padding-right: 2%; border-right: 1px solid #999; vertical-align:top;"]]\n++ Recent edits (all wikis)\n[[module RecentWRevisions]]\n[[/cell]]\n[[cell style="width: 45%; padding-left: 2%; vertical-align:top;"]]\n++ Top Sites\n[[module MostActiveSites]]\n++ Top Forums\n[[module MostActiveForums]]\n++ New users\n[[module NewWUsers]]\n++ Some statistics\n[[module SomeGlobalStats]]\n[[/cell]]\n[[/row]]\n[[/table]]
108	[!--\nSearch Users\n--]\nTo look for someone, please enter:\n\n* email address of a person you are looking for (this will look for exact match)\n* any part of the screen name or realname (lists all Users matching the query)\n\n[[module SearchUsers]]
109	[!--\nSearch This Wiki\n--]\n[[module Search]]
92	* [# Sample Menu]\n * [[[mywikidot-info|Experienced users]]]\n * [[[mywikidot-blank|Link to a non-existing page]]]\n* [# Edit/Print]\n * [[button edit text="Edit This Page"]]\n * [[button print text="Print This Page"]]\n* [# Admin]\n * [[[nav:top|Edit Top Navigation]]]\n * [[[nav:side|Edit Side Navigation]]]\n * [[[admin:manage|Site Manager]]]\n\n[!-- top nav menu, use only one bulleted list above --]
86	+++ [[[blogstart|MyBlog]]]\n[[module PageCalendar category="blog" urlAttrPrefix="199" startPage="blogstart"]]\n----\n**Add a Blog Entry**\n[[module NewPage size="15" button="new entry" category="blog"]]\n----\n* [[[start | Welcome page]]]\n\n* [[[What is a Wiki Site?]]]\n* [[[How to edit pages?]]]\n\n* [[[system: join | How to join this site?]]]\n* [[[system:members | Site members]]] \n\n* [[[system: Recent changes]]]\n* [[[system: List all pages]]]\n* [[[system:page-tags-list|Page Tags]]]\n\n* [[[admin:manage|Site Manager]]]\n\n++ Page tags\n[[module TagCloud minFontSize="80%" maxFontSize="200%"  maxColor="8,8,64" minColor="100,100,128" target="system:page-tags" limit="30"]]\n\n++ Add a new page\n[[module NewPage size="15" button="new page"]]\n\n= [[size 80%]][[[nav:side | edit this panel]]][[/size]]
74	[[module SiteChanges]]
50	According to [http://en.wikipedia.org/wiki/Wiki Wikipedia], the world largest wiki site:\n\n> A //Wiki// ([ˈwiː.kiː] <wee-kee> or [ˈwɪ.kiː] <wick-ey>) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.\n\nAnd that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.
87	++ Blog Template\nThis template is designed to get your blog site up and running as quickly as possible. Don't be concerned about the error in your side navigations:\n||~ The requested categories _\ndo not (yet) exist.||\nOnce you create your first blog entry it will go away!\n\n++ If this is your first site\n\nThen there are some things you need to know:\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org].
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
88	[[module ListPages category="blog" date="@URL" separate="true" urlAttrPrefix="199"]]\n++ %%linked_title%%\n##gray|created: %%date|%e %B %Y, %H:%M%% ##\n%%content%%\n[[/module]]\n----
89	[[module ListPages category="blog" date="@URL" separate="true"]]\n++ %%linked_title%%\n##gray|created: %%date|%e %B %Y, %H:%M%% ##\n%%content%%\n[[/module]]\n----
65	* [[[start | Welcome page]]]\n\n* [[[What is a Wiki Site?]]]\n* [[[How to edit pages?]]]\n\n* [[[system: join | How to join this site?]]]\n* [[[system:members | Site members]]] \n\n* [[[system: Recent changes]]]\n* [[[system: List all pages]]]\n* [[[system:page-tags-list|Page Tags]]]\n\n* [[[admin:manage|Site Manager]]]\n\n++ Page tags\n[[module TagCloud minFontSize="80%" maxFontSize="200%"  maxColor="8,8,64" minColor="100,100,128" target="system:page-tags" limit="30"]]\n\n++ Add a new page\n[[module NewPage size="15" button="new page"]]\n\n= [[size 80%]][[[nav:side | edit this panel]]][[/size]]
66	* [# example menu]\n * [[[submenu]]]\n* [[[contact]]]\n\n[!-- top nav menu, use only one bulleted list above --]
67	[[module Search]]\n\n[!-- please do not remove or change this page if you want to keep the search function working --]
68	According to [http://en.wikipedia.org/wiki/Wiki Wikipedia], the world largest wiki site:\n\n> A //Wiki// ([ˈwiː.kiː] <wee-kee> or [ˈwɪ.kiː] <wick-ey>) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.\n\nAnd that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.
69	++ If this is your first site\n\nThen there are some things you need to know:\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org].
75	+ Members:\n\n[[module Members]]\n\n+ Moderators\n\n[[module Members group="moderators"]]\n\n+ Admins\n\n[[module Members group="admins"]]
76	[[div style="float:right; width: 50%;"]]\n[[module TagCloud limit="200" target="system:page-tags"]]\n[[/div]]\n[[module PagesByTag]]\n\n[!--\n\nYou can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module \nBut if you want to keep the tag functionality working - do not remove these modules.\n\n--]
77	[[module Pages preview="true"]]
78	* [[[system:join|Join This Site]]]\n* [# Edit/Print]\n * [[button edit text="Edit This Page"]]\n * [[button print text="Print This Page"]]\n* [# Admin]\n * [[[nav:top|Edit Top Navigation]]]\n * [[[nav:side|Edit Side Navigation]]]\n * [[[admin:manage|Site Manager]]]\n\n[!-- top nav menu, use only one bulleted list above --]
114	Test
117	Test, new revision
118	Test
119	Test, new revision 3.
120	Test, new revision 4.
121	Test, new revision 5.
122	Test, new revision 6.
123	Test, new revision 7.
124	Test, new revision 8.
125	Test, new revision 9.
126	Test, new revision 10.
128	Test, new revision 11.
129	Test, new revision 12.
130	Test, new revision 13.
131	Test, new revision 14.
132	Test, new revision 15.
133	Test, new revision 16.
136	Test, new revision 17.
137	Test, new revision 18.
138	Test, new revision 19.
139	**Test, new revision 19.**
140	**Test, new revision 20.**
141	**Test, new revision 21.**
142	**Test, new revision 22.**
143	**Test, new revision 23.**
144	**Test,** new revision 23.
145	**Test,** new revision 24.
146	This is the custom theme file for the SCP Wiki.\n\n----\n\n> + Instructions for SCP Translation Sites\n>\n> To use this theme for your Wikidot site, please follow these instructions:\n> # Copy all files attached to this page.\n> # Create a page named "component:theme" on your site. You may wish to lock this page.\n> # Upload all attached files to that page.\n> # Copy the code on this page to your "component:theme" page. Rename all "scp-wiki.net" instances with your Wikidot site URL.\n> # In the Site Manager for your site, go to **Appearances & Behavior**, **Themes**, then select the **External** tab, and enter @@"http://your-site-name.wikidot.com/component:theme/code/1"@@.\n> # Copy the [[[nav:side|Side Bar]]] and/or [[[nav:top|Top Bar]]] if you wish to use the new format for those. Note that the bullet icons are attached to the side bar page as well.\n> # Copy the [[[main|Front Page]]] template, including attached files.\n> # To activate the mobile view, go to any page on your wiki and click "+ Options" at the bottom.\n> # Click "Edit Meta"\n> # Click "Add a new meta tag". For name, use "viewport" (without quotes) and for content use "width=device-width, initial-scale=1.0" (again, without quotes).\n> # Profit!\n\n----\n\n[[code type="css"]]\n@charset "utf-8";\n\n@import url('http://www.scp-wiki.net/local--files/component:theme/font-bauhaus.css');\n@import url('http://fonts.googleapis.com/earlyaccess/nanumgothic.css');\n@import url('http://fonts.googleapis.com/earlyaccess/nanumgothiccoding.css');\n@import url('http://maxcdn.bootstrapcdn.com/font-awesome/4.3.0/css/font-awesome.min.css');\n\n/* SCP Sigma 9 [2014 Wikidot Theme] Created for the SCP Foundation by Aelanna Edited for SCP Foundation by Dr Devan */\n/* COMMON */\n #content-wrap {\n     position: relative;\n     margin: 2em auto 0;\n     max-width: 1040px;\n     min-height:1300px;\n     height: auto !important;\n     height: 1500px;\n}\n h1, #page-title {\n     color: #901;\n     padding: 0 0 0.25em;\n     margin: 0 0 0.6em;\n     font-weight: normal;\n}\n h1 {\n     margin-top: 0.7em;\n     padding: 0;\n     font-weight: bold;\n}\n h2, h3, h4, h5, h6, h7 {\n     margin: 0.5em 0 0.4em;\n     padding: 0;\n     letter-spacing: 1px;\n}\n #page-title {\n     border-color: #bbb;\n}\n ul {\n    /* list-style-image: url(http://d3g0gp89917ko0.cloudfront.net/v--3b8418686296/common--theme/shiny/images/bullet.gif);\n     */\n     list-style: square;\n}\n #top-bar ul {\n    /* list-style-image: none;\n    /* IE ONLY! IF list-style-image IS SET */\n}\n li, p {\n     line-height: 141%;\n}\n a {\n     color: #b01;\n     text-decoration: none;\n     background: transparent;\n}\n #side-bar a:visited {\n     color: #b01;\n}\n a:visited {\n     color: #824;\n}\n a.newpage {\n     color: #d61;\n     text-decoration: none;\n     background: transparent;\n}\n a:hover {\n     text-decoration: underline;\n     background-color: transparent;\n}\n .form-control {\n     width: 95%;\n}\n/* GLOBAL WIDTH */\n #header, #top-bar {\n     width: 90%;\n     max-width: 980px;\n     margin: 0 auto;\n}\n #top-bar {\n     width: 100%;\n     margin: 0 auto;\n}\n .mobile-top-bar {\n     display: none;\n     position: absolute;\n     left: 1em;\n     bottom: 0px;\n     z-index: 0;\n}\n body {\n     background-color: #fff;\n     font-size: 0.80em;\n     color: #333;\n}\n div#container-wrap {\n     background: url(http://www.scp-wiki.net/local--files/component:theme/body_bg.png) top left repeat-x;\n}\n sup {\n     vertical-align: top;\n     position: relative;\n     top: -0.5em;\n}\n/* HEADER */\n #header {\n     height: 140px;\n     position: relative;\n     z-index: 10;\n     padding-bottom: 22px;\n    /* FOR MENU */\n     background: url(http://www.scp-wiki.net/local--files/component:theme/logo.png) 10px 40px no-repeat;\n}\n #search-top-box {\n     position: absolute;\n     top: 79px;\n     right: 9px;\n     width: 250px;\n     text-align: right;\n}\n #search-top-box-input {\n     border: solid 1px #999;\n     border-radius: 5px;\n     color: #ccc;\n     background-color: #300;\n     box-shadow: inset 1px 1px 3px rgba(0,0,0,.5);\n}\n #search-top-box-input:hover, #search-top-box-input:focus {\n     border: solid 1px #fff;\n     color: #fff;\n     background-color: #633;\n     box-shadow: inset 1px 1px 3px rgba(0,0,0,.8);\n}\n #search-top-box-form input[type=submit] {\n     border: solid 1px #999;\n     border-radius: 5px;\n     padding: 2px 5px;\n     font-size: 90%;\n     font-weight: bold;\n     color: #ccc;\n     background-color: #633;\n     background: linear-gradient(to bottom, #966,#633,#300);\n     box-shadow: 0 1px 3px rgba(0,0,0,.5);\n     cursor: pointer;\n}\n #search-top-box-form input[type=submit]:hover, #search-top-box-form input[type=submit]:focus {\n     border: solid 1px #fff;\n     color: #fff;\n     text-shadow: 0 0 1px rgba(255,255,255,.25);\n     background-color: #966;\n     background: linear-gradient(to bottom, #c99,#966,#633);\n     box-shadow: 0 1px 3px rgba(0,0,0,.8);\n}\n #login-status {\n     color: #aaa;\n     font-size: 90%;\n     z-index: 30;\n}\n #login-status a {\n     background: transparent;\n     color: #ddd;\n}\n #login-status ul a {\n     color: #700;\n     background: transparent;\n}\n #account-topbutton {\n     background: #ccc;\n     color: #700;\n}\n .printuser img.small {\n     margin-right: 1px;\n}\n #header h1 {\n     margin-left: 120px;\n     padding: 0;\n     float: left;\n     max-height: 95px;\n}\n #header h2 {\n     margin-left: 120px;\n     padding: 0;\n     clear: left;\n     float: left;\n     font-size: 105%;\n     max-height: 38px;\n}\n #header h1 a {\n     display: block;\n     margin: 0;\n     padding: 80px 0 25px;\n     line-height: 0px;\n     max-height: 0px;\n     color: #eee;\n     background: transparent;\n     font-family: BauhausLTDemi, 'Nanum Gothic', Arial, sans-serif;\n     font-size: 180%;\n     text-decoration: none;\n     text-shadow: 3px 3px 5px #000;\n     letter-spacing: 0.9px;\n}\n #header h2 span {\n     display: block;\n     margin: 0;\n     padding: 19px 0;\n     line-height: 0px;\n     max-height: 0px;\n     font-weight: bold;\n     color: #f0f0c0;\n     text-shadow: 1px 1px 1px #000;\n     text-shadow: 1px 1px 1px rgba(0,0,0,.8);\n}\n/* TOP MENU */\n #top-bar {\n     position: absolute;\n     z-index: 50;\n     top: 140px;\n     height: 21px;\n     line-height: 18px;\n     padding: 0;\n     z-index: 20;\n     font-size: 90%;\n}\n #top-bar ul {\n     float: right;\n}\n #top-bar li {\n     margin: 0;\n}\n #top-bar a {\n     color: #fff;\n     background: transparent;\n}\n #top-bar ul li {\n     border: 0;\n     position: relative;\n}\n #top-bar ul li ul {\n     border: solid 1px #666;\n     box-shadow: 0 2px 6px rgba(0,0,0,.5);\n     border-top: 0;\n}\n #top-bar ul li a {\n     border-left: solid 1px rgba(64,64,64,.1);\n     border-right: solid 1px rgba(64,64,64,.1);\n     text-decoration: none;\n     padding-top: 10px;\n     padding-bottom: 10px;\n     line-height: 1px;\n     max-height: 1px;\n     overflow: hidden;\n}\n #top-bar ul li.sfhover a, #top-bar ul li:hover a {\n     background: #eee;\n     color: #a01;\n     border-left: solid 1px rgba(64,64,64,1);\n     border-right: solid 1px rgba(64,64,64,1);\n}\n #top-bar ul li.sfhover ul li a, #top-bar ul li:hover ul li a {\n     border-width: 0;\n     width: 150px;\n     border-top: 1px solid #ddd;\n     line-height: 160%;\n     height: auto;\n     max-height: none;\n     padding-top: 0;\n     padding-bottom: 0;\n}\n #top-bar ul li.sfhover a:hover, #top-bar ul li:hover a:hover {\n     background: #fff;\n     text-decoration: none;\n}\n #top-bar ul li ul {\n     border-width: 0 1px 1px 1px;\n     width: auto;\n}\n #top-bar ul li ul li, #top-bar ul li ul li.sfhover, #top-bar ul li ul li, #top-bar ul li ul li:hover {\n     border-width: 0;\n}\n #top-bar ul li ul li a {\n     border-width: 0;\n}\n #top-bar ul li ul a, #top-bar a:hover {\n     color: #a01;\n}\n .top-bar ul li:last-of-type ul {\n     right: 0;\n}\n/* IE7 HACK */\n #top-bar ul > li > ul {\n     *margin-top: -4px;\n}\n/* SIDE MENU */\n #side-bar {\n     clear: none;\n     float: none;\n     position: absolute;\n     top: 0.5em;\n     left: 2em;\n     width: 17em;\n     padding: 0;\n     border: none;\n     display: block;\n}\n #side-bar .side-block {\n     padding: 10px;\n     border: 1px solid #660000;\n     border-radius: 10px;\n     box-shadow: 0 2px 6px rgba(102,0,0,.5);\n     background: #fff;\n     margin-bottom: 15px;\n}\n #side-bar .side-area {\n     padding: 10px;\n}\n #side-bar .heading {\n     color: #600;\n     border-bottom: solid 1px #600;\n     padding-left: 15px;\n     margin-top: 10px;\n     margin-bottom: 5px;\n     font-size: 8pt;\n     font-weight: bold;\n}\n #side-bar p {\n     margin: 0;\n}\n #side-bar div.menu-item {\n     margin: 2px 0;\n}\n #side-bar div.menu-item img {\n     width: 13px;\n     height: 13px;\n     border: 0;\n     margin-right: 2px;\n     position: relative;\n     bottom: -2px;\n}\n #side-bar div.menu-item a {\n     font-weight: bold;\n}\n #side-bar div.menu-item.inactive img {\n     opacity: 0.25;\n}\n #side-bar div.menu-item.inactive a {\n     color: #999;\n}\n #side-bar div.menu-item .sub-text {\n     font-size: 80%;\n     color: #666;\n}\n #side-bar div.menu-item.sub-item {\n}\n #side-bar .collapsible-block-folded {\n     background: url(http://www.scp-wiki.net/local--files/nav:side/expand.png) 0 2px no-repeat;\n}\n #side-bar .collapsible-block-link {\n     margin-left: 15px;\n     font-weight: bold;\n}\n #side-bar .collapsible-block-unfolded-link {\n     border-bottom: solid 1px #600;\n}\n #side-bar .collapsible-block-unfolded-link .collapsible-block-link {\n     margin-top: 10px;\n     margin-bottom: 5px;\n     font-size: 8pt;\n     color: #600;\n}\n #side-bar .collapsible-block-unfolded-link .collapsible-block-link:hover {\n     color: #b01;\n     text-decoration: none;\n}\n #side-bar ul{\n     list-style-type: none;\n     padding: 0 5px 0;\n}\n/* CONTENT */\n #main-content {\n     margin: 0 2em 0 22em;\n     padding: 1em;\n     position: relative;\n}\n/* ACTUALLY HIDE HIDDEN TAGS */\n #main-content .page-tags a[href^="/system:page-tags/tag/_"] {\n     display: none;\n}\n #breadcrumbs {\n     margin: -1em 0 1em;\n     font-weight: 85%;\n}\n/* YUI-TABS */\n .yui-navset .yui-content{\n     background-color: #f5f5f5;\n}\n .yui-navset .yui-nav a, .yui-navset .yui-navset-top .yui-nav a {\n     background-color:#d8d8d8;\n     background-image: url(http://d3g0gp89917ko0.cloudfront.net/v--3b8418686296/common--theme/shiny/images/yuitabs.png);\n}\n .yui-navset .yui-nav .selected a, .yui-navset .yui-nav .selected a:focus,\n/* no focus effect for selected */\n .yui-navset .yui-nav .selected a:hover {\n    /* no hover effect for selected */\n     background:#700 url(http://d3g0gp89917ko0.cloudfront.net/v--3b8418686296/common--theme/shiny/images/yuitabs.png) repeat-x left -1400px;\n    /* selected tab background */\n     color:#fff;\n}\n .yui-navset .yui-nav a:hover, .yui-navset .yui-nav a:focus {\n     background:#d88 url(http://d3g0gp89917ko0.cloudfront.net/v--3b8418686296/common--theme/shiny/images/yuitabs.png) repeat-x left -1300px;\n     text-decoration: none;\n}\n .yui-navset .yui-nav, .yui-navset .yui-navset-top .yui-nav {\n     border-color: #444;\n}\n .yui-navset .yui-nav, .yui-navset .yui-navset-top .yui-nav {\n     border-color: #444;\n}\n .yui-navset li {\n     line-height: normal;\n}\n/* FOOTER */\n #footer {\n     clear: both;\n     font-size: 77%;\n     background: #444;\n     color: #bbb;\n     margin-top: 15px;\n     padding: 3px 10px;\n}\n #footer .options {\n     visibility: visible;\n     display: block;\n     float: right;\n     width: 50%;\n     font-size: 100%;\n     text-align: right;\n}\n #footer a {\n     color: #fff;\n     background: transparent;\n}\n/* SOME NICE BOXES */\n div.sexy-box {\n     background: #eee;\n     border: 1px solid #ccc;\n     padding: 0 10px 12px;\n     margin: 7px 4px 12px;\n     overflow: hidden;\n}\n div.sexy-box div.image-container img {\n     margin: 5px;\n     padding: 2px;\n     border: 1px solid #999;\n}\n/* Custom page content classes */\n #page-content {\n     min-height: 720px;\n}\n .unmargined > p {\n     margin: 0;\n     line-height: 100%;\n}\n .content-panel {\n     border: solid 1px #888880;\n     border-radius: 10px;\n     background-color: #999990;\n     margin: 10px 0 15px;\n     box-shadow: 3px 3px 6px #bbb;\n     box-shadow: 0 2px 6px rgba(0,0,0,0.5), inset 0 1px rgba(255,255,255,0.3), inset 0 10px rgba(255,255,255,0.2), inset 0 10px 20px rgba(255,255,255,0.25), inset 0 -15px 30px rgba(0,0,0,0.1);\n}\n .content-panel.standalone {\n     background: #fcfdfb;\n}\n .content-panel.series {\n     padding: 0 20px;\n     margin-bottom: 20px;\n}\n .content-panel.centered {\n     text-align: center;\n}\n .content-panel.left-column {\n     float: left;\n     width: 48%;\n}\n .content-panel.right-column {\n     float: right;\n     width: 48%;\n}\n .content-panel .panel-heading {\n     padding: 2px 10px;\n     color: #ffffff;\n     font-size: 90%;\n     font-weight: bold;\n     text-shadow: 1px 1px 2px rgba(0,0,0,.35);\n}\n .content-panel .panel-heading > p, .content-panel .panel-footer > p {\n     margin: 0;\n}\n .content-panel .panel-body {\n     padding: 5px 10px;\n     background: #fff9f0 url(http://www.scp-wiki.net/local--files/component:theme/panel-bg-gradient-reverse.png) bottom repeat-x;\n    /* box-shadow: 1px 2px 3px rgba(0,0,0,.2) inset */\n}\n .content-panel .panel-footer {\n     padding: 1px 10px;\n     color: #fffff0;\n     font-size: 80%;\n     font-weight: bold;\n     text-align: right;\n     text-shadow: 1px 1px 2px rgba(0,0,0,.5);\n}\n .content-panel .panel-footer a {\n     color: #ffffff;\n}\n .content-panel .content-toc {\n     float: right;\n     padding: 0 20px;\n     background-color: #fff;\n     border: solid 1px #ccc;\n     border-radius: 10px;\n     margin: 20px 0 5px 5px;\n     white-space: nowrap;\n     box-shadow: inset 1px 2px 6px rgba(0,0,0,.15)\n}\n .alternate:nth-child(even) {\n     background-color: rgba(255,255,255,.9);\n}\n/* Page Rating Module Customizations */\n .page-rate-widget-box {\n     display: inline-block;\n     border-radius: 5px;\n     box-shadow: 1px 1px 3px rgba(0,0,0,.5);\n     margin-bottom: 10px;\n     margin-right: 2em;\n}\n .page-rate-widget-box .rate-points {\n     background-color: #633 !important;\n     border: solid 1px #633;\n     border-right: 0;\n     border-radius: 5px 0 0 5px;\n}\n .page-rate-widget-box .rateup, .page-rate-widget-box .ratedown {\n     background-color: #fff6f0;\n     border-top: solid 1px #633;\n     border-bottom: solid 1px #633;\n     font-weight: bold;\n}\n .page-rate-widget-box .rateup a, .page-rate-widget-box .ratedown a {\n     background: transparent;\n     color: #633;\n     padding: 0 4px;\n     margin: 0 1px;\n}\n .page-rate-widget-box .rateup a:hover, .page-rate-widget-box .ratedown a:hover {\n     background: #633;\n     color: #fffff0;\n     text-decoration: none;\n}\n .page-rate-widget-box .cancel {\n     background-color: #633;\n     border: solid 1px #633;\n     border-left: 0;\n     border-radius: 0 5px 5px 0;\n}\n .page-rate-widget-box .cancel a {\n     background: transparent;\n     text-transform: uppercase;\n     color: #966;\n}\n .page-rate-widget-box .cancel a:hover {\n     border-radius: 0 3px 3px 0;\n     background: #633;\n     color: #fffff0;\n     text-decoration: none;\n}\n/* Heritage Collection Rating Module */\n .heritage-rating-module {\n     display: inline-block;\n     background-color: #633;\n     padding: 2px 8px 2px 5px;\n     border: solid 1px #ccc066;\n     border-radius: 8px;\n     box-shadow: 0 1px 3px rgba(0,0,0,.25);\n     margin-right: 2em;\n     margin-bottom: 10px;\n}\n .heritage-rating-module .page-rate-widget-box {\n     box-shadow: none;\n     margin-bottom: 0;\n     margin-right: 0;\n}\n .heritage-rating-module .heritage-emblem {\n     float: left;\n     position: relative;\n     top: -2px;\n     left: 2px;\n     height: 16px;\n     width: 16px;\n     overflow: visible;\n     margin-right: 2px;\n}\n .heritage-rating-module .heritage-emblem img {\n     width: 20px;\n     height: 20px;\n     border: 0;\n}\n/* Fixes for multi-line page tags */\n #main-content .page-tags {\n     margin: 1em 0 0;\n     padding: 0;\n}\n #main-content .page-tags span {\n     display: inline-block;\n     padding: 0;\n     max-width: 60%;\n}\n #main-content .page-tags a {\n     display: inline-block;\n     white-space: nowrap;\n}\n/* Standard Image Block */\n .scp-image-block {\n     border: solid 1px #666;\n     box-shadow: 0 1px 6px rgba(0,0,0,.25);\n     width: 300px;\n}\n .scp-image-block.block-right {\n     float: right;\n     clear: right;\n     margin: 0 2em 1em 2em;\n}\n .scp-image-block.block-left {\n     float: left;\n     clear: left;\n     margin: 0 2em 1em 0;\n}\n .scp-image-block.block-center {\n     margin-right: auto;\n     margin-left: auto;\n}\n .scp-image-block img {\n     border: 0;\n     width: 300px;\n}\n .scp-image-block .scp-image-caption {\n     background-color: #eee;\n     border-top: solid 1px #666;\n     padding: 2px 0;\n     font-size: 80%;\n     font-weight: bold;\n     text-align: center;\n     width: 300px;\n}\n .scp-image-block > p {\n     margin: 0;\n}\n .scp-image-block .scp-image-caption > p {\n     margin: 0;\n     padding: 0 10px;\n}\n/* Wikiwalk Navigation */\n .footer-wikiwalk-nav {\n     font-weight: bold;\n     font-size: 75%;\n}\n/* Forum Customizations */\n .forum-thread-box .description-block {\n     padding: .5em 1em;\n     border-radius: 10px;\n     box-shadow: 0 1px 5px rgba(0,0,0,.15), inset 0 1px 0 rgba(255,255,255,.8), inset 0 10px 5px rgba(255,255,255,.25), inset 0 -15px 30px rgba(0,0,0,.1)\n}\n .thread-container .post .head {\n     padding: 0.5em 1em;\n     background-color: #eee;\n     background: linear-gradient(to right, #eee, #eeecec);\n     box-shadow: inset 2px 3px 6px rgba(0,0,0,.15);\n     border-radius: 5px 5px 0 0;\n}\n/* Hide Forum Signatures */\n .signature {\n     display:none !important;\n}\n/* Ruby by Nanimono Demonai */\n .ruby, ruby {\n     display: inline-table;\n     text-align: center;\n     white-space: nowrap;\n     line-height: 1;\n     height: 1em;\n     vertical-align: text-bottom;\n}\n .rt, rt {\n     display: table-header-group;\n     font-size: 0.6em;\n     line-height: 1.1;\n     text-align: center;\n     white-space: nowrap;\n}\n/* Keycap */\n .keycap {\n     border: 1px solid;\n     border-color: #ddd #bbb #bbb #ddd;\n     border-bottom-width: 2px;\n     -moz-border-radius: 3px;\n     -webkit-border-radius: 3px;\n     border-radius: 3px;\n     background-color: #f9f9f9;\n     padding: 1px 3px;\n     font-family: inherit;\n     font-size: 0.85em;\n     white-space: nowrap;\n}\n/* tag style */\n .tags {\n     display: inline-block;\n     margin: 0 0 0 5px;\n     padding: 3px 5px 3px 0px;\n     height: 13px;\n     line-height: 13px;\n     font-size: 11px;\n     background: #666;\n     color: #fff;\n     text-decoration: none;\n     -moz-border-radius-bottomright: 4px;\n     -webkit-border-bottom-right-radius: 4px;\n     border-bottom-right-radius: 4px;\n     -moz-border-radius-topright: 4px;\n     -webkit-border-top-right-radius: 4px;\n     border-top-right-radius: 4px;\n}\n .tags:before {\n     content: "";\n     padding: 0px 1px 3px 1px;\n     float: left;\n     position: relative;\n     top: -3px;\n     left: -10px;\n     width: 0;\n     height: 0;\n     border-color: transparent #666 transparent transparent;\n     border-style: solid;\n     border-width: 8px 8px 8px 0px;\n}\n .tags:after {\n     content: "";\n     position: relative;\n     top: 4.5px;\n     left: -8px;\n     float: left;\n     width: 4px;\n     height: 4px;\n     -moz-border-radius: 2px;\n     -webkit-border-radius: 2px;\n     border-radius: 2px;\n     background: #fff;\n     -moz-box-shadow: -1px -1px 2px #004977;\n     -webkit-box-shadow: -1px -1px 2px #333;\n     box-shadow: -1px -1px 2px #333;\n}\n/* Display Black Block by Nanimono Demonai */\n .bblock {\n     color: #000000;\n     background-color:#000000;\n     transition: 2s;\n     text-decoration: none;\n}\n .bblock:hover {\n     background-color:#000000;\n     color: #006600;\n     text-decoration: none;\n}\n .dblock {\n     color:#000000;\n     background-color:#000000;\n     transition: 2s;\n     text-decoration: none;\n}\n .dblock:hover {\n     background-color:transparent;\n     text-decoration: none;\n}\n/* Blockquote Mimic Div */\n div.blockquote {\n     border: 1px dashed #999;\n     background-color: #f4f4f4;\n     padding: 0 1em;\n     margin: 1em 3em;\n}\n @media (max-width: 479px) {\n     div.blockquote {\n         margin: 1em 0;\n    }\n}\n @media (min-width: 480px) and (max-width: 580px) {\n     div.blockquote {\n         margin: 0.5em;\n    }\n}\n/* 2011-11-13 Minobe Hiroyuki @ Marguerite Site www.marguerite.jp/Nihongo/WWW/CSSTips/EmphasizeDots-CSS3.html Edited for the SCP Foundation by Nanimono_Demonai */\n .emph {\n     text-emphasis-style: dot;\n     -webkit-text-emphasis-style: dot;\n}\n/* For FireFox */\n @-moz-document url-prefix() {\n     .emph {\n        /* For the environments which comply with CSS3. */\n         font-family: monospace;\n         font-style: normal;\n         font-weight: normal;\n         background-image: url(http://www.scp-wiki.net/local--files/component%3Atheme/dot.png), none;\n         background-repeat: repeat-x;\n         padding: 0.5em 0 0;\n         background-color:transparent;\n         background-clip: padding-box, content-box;\n         background-size: 1em 1.3em, auto;\n    }\n}\n/* For IE10 */\n @media screen and (-ms-high-contrast: active), (-ms-high-contrast: none) {\n     .emph {\n        /* For the environments which comply with CSS3. */\n         font-family: monospace;\n         font-style: normal;\n         font-weight: normal;\n         background-image: url(http://www.scp-wiki.net/local--files/component%3Atheme/dot.png), none;\n         background-repeat: repeat-x;\n         padding: 0.5em 0 0;\n         background-color:transparent;\n         background-clip: padding-box, content-box;\n         background-size: 1em 1.3em, auto;\n    }\n}\n/* viewport */\n @viewport {\n     width: device-width;\n     zoom: 1.0;\n}\n/* IE viewport */\n @-ms-viewport {\n     width: device-width;\n     zoom: 1.0;\n}\n/* opera viewport */\n @-o-viewport {\n     width: device-width;\n     zoom: 1.0;\n}\n/* chrome viewport - maybe it isn't work... */\n @-webkit-viewport {\n     width: device-width;\n     zoom: 1.0;\n}\n/* firefox viewport - maybe it isn't work too... */\n @-moz-viewport {\n     width: device-width;\n     zoom: 1.0;\n}\n/* webkit scrollbar */\n ::-webkit-scrollbar {\n     width: 9px;\n    /* for vertical scrollbars */\n     height: 9px;\n    /* for horizontal scrollbars */\n     border: solid 1px rgba(0, 0, 0, 0.1);\n     border-round: 0.5px;\n}\n ::-webkit-scrollbar-track {\n     background: rgba(0, 0, 0, 0.1);\n}\n ::-webkit-scrollbar-thumb {\n     background: rgba(50, 50, 50, 0.3);\n}\n .page-source {\n     word-break: break-all;\n}\n/* Responsive Web Design */\n img, embed, video, object, iframe, table {\n     max-width: 100%;\n}\n #page-content div, #page-content div table {\n     max-width: 100%;\n}\n #edit-page-comments {\n     width: 100%;\n}\n/* basic Query for mobile devices */\n @media (max-width: 767px) {\n     .owindow {\n         min-width: 80%;\n         max-width: 99%;\n    }\n     .modal-body .table, .modal-body .table ~ div {\n         float: left;\n    }\n     .owindow .button-bar {\n         float: right;\n    }\n     .owindow div a.btn-primary {\n         width: 100%;\n         float: left;\n    }\n     .mobile-top-bar ul li:last-of-type ul {\n         right: 0;\n    }\n     span, a {\n         word-break: break-all;\n    }\n}\n/* Mobile Media Query */\n @media (max-width: 479px) {\n     #search-top-box-input {\n         display: none;\n    }\n     #page-content {\n         font-size: 0.9em;\n    }\n     #main-content {\n         margin: 0;\n    }\n     #recent-posts-category {\n         width: 100%;\n    }\n     #header, .mobile-top-bar {\n         max-width: 90%;\n    }\n     #side-bar {\n         width: 80%;\n         position: relative;\n    }\n     .top-bar {\n         display:none;\n    }\n     .mobile-top-bar {\n         display: block;\n         padding: 0;\n    }\n     .page-options-bottom a {\n         padding: 0 4px;\n    }\n     #header h1 a {\n         font-size: 100%;\n    }\n     blockquote {\n         margin: 1em 0;\n    }\n     .license-area {\n         font-size: 0.8em;\n    }\n     #header {\n         background-position: 0 5.5em;\n         background-size: 55px 55px;\n    }\n     #header h1, #header h2 {\n         margin-left: 66px;\n    }\n     table.form td, table.form th {\n         float: left;\n    }\n    /* td.title {\n         width: 30%;\n    }\n     */\n     td.name {\n         width: 15em;\n    }\n     table.form td, table.form th {\n         padding: 0;\n    }\n     #edit-page-title {\n         max-width: 90%;\n    }\n     .content-panel.left-column, .content-panel.right-column {\n         width: 99%;\n         float: left;\n    }\n     #page-content div, #page-content div table {\n         clear: both;\n    }\n     #page-content div.title {\n         word-break: keep-all;\n    }\n}\n/* Small Mobile Media Query */\n @media (max-width: 385px) {\n     #header {\n         background-position: 5% 5.5em;\n    }\n     #header h1, #header h2 {\n         margin-left: -webkit-calc(66px + 5%);\n         margin-left: -moz-calc(66px + 5%);\n         margin-left: calc(66px + 5%);\n    }\n     #header, #top-bar {\n         width: 100%;\n         max-width: 100%;\n    }\n     .mobile-top-bar {\n         width: 100%;\n    }\n     #top-bar li a {\n         padding: 10px 0.5em;\n    }\n}\n/* Note Media Query */\n @media (min-width: 480px) and (max-width: 580px) {\n     #search-top-box-input {\n         width: 7em;\n    }\n     #main-content {\n         margin: 0 2em 0 2em;\n    }\n     #header, .mobile-top-bar {\n         max-width: 90%;\n    }\n     #side-bar {\n         width: 80%;\n         position: relative;\n    }\n     .top-bar {\n         display:none;\n    }\n     .mobile-top-bar {\n         display: block;\n    }\n     .page-options-bottom a {\n         padding: 0 5px;\n    }\n     #header h1 a {\n         font-size: 120%;\n    }\n     blockquote {\n         margin: 0.5em;\n    }\n     .license-area {\n         font-size: 0.85em;\n    }\n     #header {\n         background-position: 0.5em 4.5em;\n         background-size: 66px 66px;\n    }\n     #header h1, #header h2 {\n         margin-left: 80px;\n    }\n    /* td.title {\n         width: 30%;\n    }\n     */\n     #page-content div.title {\n         word-break: keep-all;\n    }\n     td.name {\n         width: 15em;\n    }\n     .content-panel.left-column, .content-panel.right-column {\n         width: 99%;\n         float: left;\n    }\n     #page-content div, #page-content div table {\n         clear: both;\n    }\n}\n/* Mini Tablet Media Query */\n @media (min-width: 581px) and (max-width: 767px) {\n     #search-top-box-input {\n         width: 8em;\n    }\n     #side-bar {\n         width: 80%;\n         position: relative;\n    }\n     #main-content {\n         margin: 0 3em 0 2em;\n    }\n     #header, .mobile-top-bar {\n         max-width: 90%;\n    }\n     .top-bar {\n         display: none;\n    }\n     .mobile-top-bar {\n         display: block;\n    }\n     .page-options-bottom a {\n         padding: 0 6px;\n    }\n     #header h1 a {\n         font-size: 140%;\n    }\n     .license-area {\n         font-size: 0.9em;\n    }\n     #header {\n         background-position: 1em 4em;\n         background-size: 77px 77px;\n    }\n     #header h1, #header h2 {\n         margin-left: 93px;\n    }\n}\n/* Tablet Media Query */\n @media (min-width: 768px) and (max-width: 979px) {\n     #main-content {\n         margin: 0 4em 0 20em;\n    }\n     #header, #top-bar #side-bar {\n         max-width: 100%;\n    }\n     .top-bar li {\n         margin: 0;\n    }\n     #top-bar ul li.sfhover ul li a, #top-bar ul li:hover ul li a {\n         width: 110px;\n    }\n     .page-options-bottom a {\n         padding: 0 7px;\n    }\n     #header h1 a {\n         font-size: 160%;\n    }\n     .license-area {\n         font-size: 0.95em;\n    }\n     #header {\n         background-position: 1em 4em;\n         background-size: 88px 88px;\n    }\n     #header h1, #header h2 {\n         margin-left: 106px;\n    }\n     .content-panel.left-column, .content-panel.right-column {\n         width: 99%;\n         float: left;\n    }\n     #page-content div, #page-content div table {\n         clear: both;\n    }\n}\n/* Desktop Media Query ----------- @media (min-width: 980px) and (max-width: 1399px) {\n}\n ------------------------------------------ */\n/* Wide Monitor Media Query ----- @media (min-width: 1400px) {\n}\n ------------------------------------------ */\n/* off-canvas */\n .close-menu {\n     display: none;\n}\n @media (max-width: 767px) {\n     .page-history tbody tr td:last-child {\n         width: 35%;\n    }\n     .owindow {\n         min-width: 80%;\n         max-width: 99%;\n    }\n     .modal-body .table, .modal-body .table ~ div {\n         float: left;\n    }\n     .owindow .button-bar {\n         float: right;\n    }\n     .owindow div .btn-primary {\n         width: 100%;\n         float: left;\n    }\n     .owindow div .btn-primary ~ div {\n         width: 100%;\n    }\n     .yui-navset {\n         z-index: 1;\n    }\n     #navi-bar, #navi-bar-shadow {\n         display: none;\n    }\n     .open-menu a {\n         position: fixed;\n         top: 0.5em;\n         left: 0.5em;\n         z-index: 15;\n         font-family: 'Nanum Gothic', san-serif;\n         font-size: 30px;\n         font-weight: 700;\n         width: 30px;\n         height: 30px;\n         line-height: 0.9em;\n         text-align: center;\n         border: 0.2em solid #888 !important;\n         background-color: #fff !important;\n         border-radius: 3em;\n         color: #888 !important;\n    }\n     .open-menu a:hover {\n         text-decoration: none !important;\n         -webkit-box-shadow: 0px 0px 20px 3px rgba(153,153,153,1);\n         -moz-box-shadow: 0px 0px 20px 3px rgba(153,153,153,1);\n         -ms-box-shadow: 0px 0px 20px 3px rgba(153,153,153,1);\n         -o-box-shadow: 0px 0px 20px 3px rgba(153,153,153,1);\n         box-shadow: 0px 0px 20px 3px rgba(153,153,153,1);\n    }\n     #main-content {\n         max-width: 90%;\n         margin: 0 5%;\n         padding: 0;\n         -webkit-transition: 0.5s ease-in-out 0.1s;\n         -moz-transition: 0.5s ease-in-out 0.1s;\n         -ms-transition: 0.5s ease-in-out 0.1s;\n         -o-transition: 0.5s ease-in-out 0.1s;\n         transition: 0.5s ease-in-out 0.1s;\n    }\n     #side-bar {\n         display: block;\n         position: fixed;\n         top: 0;\n         left: -25em;\n         width: 17em;\n         height: 100%;\n         background-color: rgb(184, 134, 134);\n         overflow-y: auto;\n         z-index: 10;\n         padding: 1em 1em 0 1em;\n         -webkit-transition: left 0.5s ease-in-out 0.1s;\n         -moz-transition: left 0.5s ease-in-out 0.1s;\n         -ms-transition: left 0.5s ease-in-out 0.1s;\n         -o-transition: left 0.5s ease-in-out 0.1s;\n         transition: left 0.5s ease-in-out 0.1s;\n    }\n     #side-bar:after {\n         content: "";\n         position: absolute;\n         top: 0;\n         width: 0;\n         height: 100%;\n         background-color: rgba(0, 0, 0, 0.2);\n    }\n     #side-bar:target {\n         display: block;\n         left: 0;\n         width: 17em;\n         margin: 0;\n         border: 1px solid #dedede;\n         z-index: 10;\n    }\n     #side-bar:target + #main-content {\n         left: 0;\n    }\n     #side-bar:target .close-menu {\n         display: block;\n         position: fixed;\n         width: 100%;\n         height: 100%;\n         top: 0;\n         left: 0;\n         background: rgba(0,0,0,0.3) 1px 1px repeat;\n         z-index: -1;\n    }\n}\n div.scpnet-interwiki-wrapper {\n     width: 17em;\n     margin-left: -5px;\n}\n iframe.scpnet-interwiki-frame {\n     height: 300px;\n     width: 17em;\n     border: none;\n}\n @media (min-width: 768px) {\n     iframe.scpnet-interwiki-frame {\n         height: 300px;\n         width: 18em;\n    }\n     div.scpnet-interwiki-wrapper {\n         width: 18em;\n    }\n}\n[[/code]]
147	No profile has been set up yet for this user.
148	No profile has yet been set up.
149	No profile has yet been set up. (template:profile)
150	A profile has not been set up for this user.
151	A profile has not been set up for this user.
152	A profile has not been set up for this user.
153	* [[[start | Welcome page]]]\n\n* [[[What is a Wiki Site?]]]\n* [[[How to edit pages?]]]\n\n* [[[system: join | How to join this site?]]]\n* [[[system:members | Site members]]] \n\n* [[[system: Recent changes]]]\n* [[[system: List all pages]]]\n* [[[system:page-tags-list|Page Tags]]]\n\n* [[[admin:manage|Site Manager]]]\n\n++ Page tags\n[[module TagCloud minFontSize="80%" maxFontSize="200%"  maxColor="8,8,64" minColor="100,100,128" target="system:page-tags" limit="30"]]\n\n++ Add a new page\n[[module NewPage size="15" button="new page"]]\n\n= [[size 80%]][[[nav:side | edit this panel]]][[/size]]
154	* [# Sample Menu]\n * [[[www::start|MyWikidot Home]]]\n * [[[www::mywikidot-info|Experienced users]]]\n* [# Edit/Print]\n * [[button edit text="Edit This Page"]]\n * [[button print text="Print This Page"]]\n* [# Admin]\n * [[[nav:top|Edit Top Navigation]]]\n * [[[nav:side|Edit Side Navigation]]]\n * [[[admin:manage|Site Manager]]]\n\n[!-- top nav menu, use only one bulleted list above --]
155	[[module Search]]\n\n[!-- please do not remove or change this page if you want to keep the search function working --]
156	According to [http://en.wikipedia.org/wiki/Wiki Wikipedia], the world largest wiki site:\n\n> A //Wiki// ([ˈwiː.kiː] <wee-kee> or [ˈwɪ.kiː] <wick-ey>) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.\n\nAnd that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.
157	++ If this is your first site\n\nThen there are some things you need to know:\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org].
158	[!--\nHow To Edit Pages - Quickstart\n--]\nIf you are allowed to edit pages in this Site, simply click on //edit// button at the bottom of the page. This will open an editor with a toolbar pallette with options.\n\nTo create a link to a new page, use syntax: {{``[[[new page name]]]``}} or {{``[[[new page name | text to display]]]``}}. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!\n\nAlthough creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit [*http://www.wikidot.org/doc Documentation pages] (at wikidot.org) to learn more.
159	[[module ManageSite]]
160	[[note]]\nPlease change this page according to your policy (configure first using [[[admin:manage|Site Manager]]]) and remove this note.\n[[/note]]\n\n+ Who can join?\n\nYou can write here who can become a member of this site.\n\n+ Join!\n\nSo you want to become a member of this site? Tell us why and apply now!\n\n[[module MembershipApply]] \n\nOr, if you already know a "secret password", go for it!\n\n[[module MembershipByPassword]]
161	[[module TagCloud limit="200" target="system:page-tags"]]\n\n[!--\n\nYou can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module \nBut if you want to keep the tag functionality working - do not remove these modules.\n\n--]
162	[[module SiteChanges]]
163	+ Members:\n\n[[module Members]]\n\n+ Moderators\n\n[[module Members group="moderators"]]\n\n+ Admins\n\n[[module Members group="admins"]]
164	[[div style="float:right; width: 50%;"]]\n[[module TagCloud limit="200" target="system:page-tags"]]\n[[/div]]\n[[module PagesByTag]]\n\n[!--\n\nYou can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module \nBut if you want to keep the tag functionality working - do not remove these modules.\n\n--]
165	[[module Pages preview="true"]]
166	No profile has been set up yet for this user.
167	[[div class="side-block" style="background-color: #e5e5ff;"]]\n[[div class="menu-item"]]\n[[=]]\n[[image icon-DeviantArt-2020.png link="http://scp-foundation.deviantart.com/" style="width:30px; height:30px;"  alt="SCP DeviantArt"]][[image icon-Facebook-2020.png link="https://www.facebook.com/scpfoundation" style="width:30px; height:30px;" alt="Facebook"]][[image icon-Twitter-2020.png link="https://twitter.com/scpwiki" style="width:30px; height:30px;" alt="Twitter"]][[image icon-Reddit-2020.png link="http://www.reddit.com/r/SCP" style="width:30px; height:30px;" alt="Reddit"]][[image icon-Tumblr-2020.png link="http://scp-wiki-official.tumblr.com/" style="width:30px; height:30px;" alt="Tumblr"]][[image icon-Instagram-2020.png link="https://www.instagram.com/scpfoundationwiki/" style="width:30px; height:30px;" alt="Instagram"]]\n[[/=]]\n[[/div]]\n[[/div]]\n\n~~~~\n\n[[div class="side-block"]]\n\n\n\n[[div class="menu-item"]]\n[[image home.png]][/ Main]\n[[/div]]\n[[div class="heading"]]\nSCP by Series\n[[/div]]\n[[div class="menu-item"]]\n[[image series.png]] [[[scp-series|I]]] | [[[scp-series-2|II]]] | [[[scp-series-3|III]]] | [[[scp-series-4|IV]]] | [[[scp-series-5|V]]] | [[[scp-series-6|VI]]]  \n[[/div]]\n[[div class="heading"]]\nSCP Tales by Series\n[[/div]]\n[[div class="menu-item"]]\n[[image series.png]] [[[scp-series-1-tales-edition|I]]] | [[[scp-series-2-tales-edition|II]]] | [[[scp-series-3-tales-edition|III]]] | [[[scp-series-4-tales-edition|IV]]] | [[[scp-series-5-tales-edition|V]]]\n[[/div]]\n\n[[div class="heading"]]\nSCP Library\n[[/div]]\n[[div class="menu-item"]]\n[[image series.png]][[[foundation-tales|Tales]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image series.png]][[[canon-hub|Canons]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image series.png]][[[scp-international|International SCP Hub]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image series.png]][[[GoI Formats]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image series.png]][[[scp-ex|Explained SCPs]]]\n[[/div]]\n\n[[div class="heading"]]\nDiscover Content\n[[/div]]\n[[div class="menu-item"]]\n[[image default.png]][[[Top Rated Pages This Month| Top Rated New Pages]]]\n[[/div]]\n[[div class="menu-item sub-item"]]\n[[image default.png]][[[new-pages-feed| Newly Created Pages]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image main.png]][[[random:random-scp|Random SCP]]] | [[[random:random-tale|Tale]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image main.png]][[[system:recent-changes| Recent Changes]]] | [[[http://www.scp-wiki.net/most-recently-edited| Edits]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image default.png]][[[Lowest Rated Pages]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image main.png]][[[guide-hub|Guides & Essays]]]\n[[/div]]\n[[div class="menu-item sub-item"]]\n[[image default.png]][[[Contribute]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image main.png]][[[http://www.scp-wiki.net/young-and-under-30|Underread & Underrated]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image main.png]][[[seminars-hub|Seminars & Workshops]]]\n[[/div]]\n\n[[div class="heading"]]\nSCP Community\n[[/div]]\n[[div class="menu-item"]]\n[[image help.png]][[[Site Rules]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image main.png]][[[system:join|Join the Site!]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image forum.png]][[[forum:start|Forum]]] | [[[forum:recent-posts|New Posts]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image forum.png]][[[chat-guide|Chat With Us!]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image default.png]][[[authors-pages|Authors' Pages]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image main.png]][[[news|Site News Hub]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image default.png]][[[http://05command.wikidot.com/staff-policy-hub|Policy Hub]]]\n[[/div]]\n[[/div]]\n\n~~~~\n\n\n[[div class="side-block" style="background-color: #fff0f0;"]]\n[[div class="heading"]]\nUser Resources\n[[/div]]\n[[div class="menu-item"]]\n[[image help.png]][[[How to Write an SCP]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image default.png]][[[Tag Search]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image default.png]][[[usertools|User Tools]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image default.png]][http://www.wikidot.com/doc:start Wiki Syntax][[/div]]\n[[div class="menu-item"]]\n[[image default.png]][[[Sandbox]]][[/div]]\n[[div class="menu-item"]]\n[[image main.png]][http://05command.wikidot.com Staff Site][[/div]]\n[[div class="menu-item"]]\n[[image help.png]][[[Contact Staff]]]\n[[/div]]\n[[/div]]\n\n\n~~~~\n\n[[a href="#" class="close-menu"]]\n[[image black.png style="z-index:-1; opacity: 0.3;"]]\n[[/a]]\n\n[[div class="scpnet-interwiki-wrapper"]]\n[[module ListPages range="." limit="1"]]\n      [[iframe http://interwiki.scpdb.org/?wiki=scp-wiki&lang=en&page=%%category%%:%%name%% class="scpnet-interwiki-frame"]]\n[[/module]]\n[[/div]]
168	[[div class="top-bar"]]\n* SCP Series\n * [[[scp-series-6|Series VI]]]\n * [[[scp-series-5|Series V]]]\n * [[[scp-series-5-tales-edition|» Series V Tales]]]\n * [[[scp-series-4|Series IV]]]\n * [[[scp-series-4-tales-edition|» Series IV Tales]]]\n * [[[scp-series-3|Series III]]]\n * [[[scp-series-3-tales-edition|» Series III Tales]]]\n * [[[scp-series-2|Series II]]]\n * [[[scp-series-2-tales-edition|» Series II Tales]]]\n * [[[scp-series|Series I]]]\n * [[[scp-series-1-tales-edition|» Series I Tales]]]\n* Tales\n * [[[Foundation Tales]]]\n * [[[Series Archive]]]\n * [[[incident-reports-eye-witness-interviews-and-personal-logs|Incident Reports]]]\n * [[[Creepy-Pasta|CreepyPasta Archive]]]\n* Library\n * [[[user-curated-lists|User-Curated Lists]]]\n * [[[Joke SCPs]]]\n * [[[joke-scps-tales-edition|» Joke SCPs Tales]]]\n * [[[scp-ex|Explained SCPs]]]\n * [[[explained-scps-tales-edition|» Explained SCPs Tales]]]\n * [[[GoI Formats]]]\n * [[[Audio Adaptations]]]\n * [[[SCP Artwork Hub]]]\n * [[[Contest Archive]]]\n* Universe\n * [[[Canon Hub|Canons]]]\n * [[[Groups of Interest]]]\n * [[[log-of-anomalous-items|Anomalous Items]]]\n * [[[log-of-extranormal-events|Extranormal Events]]]\n * [[[log-of-unexplained-locations|Unexplained Locations]]]\n* SCP Global\n * [http://scp-int.wikidot.com International Translation Archive]\n * [http://scp-ru.wikidot.com Russian Branch(-RU)]\n * [http://ko.scp-wiki.net/ SCP 재단(-KO)]\n * [http://scp-wiki-cn.wikidot.com SCP基金会(-CN)]\n * [http://fondationscp.wikidot.com Fondation SCP(-FR)]\n * [http://scp-wiki.net.pl SCP Polska Filia(-PL)]\n * [http://lafundacionscp.wikidot.com La Fundación SCP(-ES)]\n * [http://scp-th.wikidot.com สถาบัน SCP(-TH)]\n * [http://scp-jp.wikidot.com SCP財団(-JP)]\n * [http://scp-wiki-de.wikidot.com SCP Deutschland(-DE)]\n * [http://fondazionescp.wikidot.com Fondazione SCP(-IT)]\n * [http://scp-ukrainian.wikidot.com Ukrainian Branch(-UA)]\n * [http://scp-pt-br.wikidot.com/ Lusófona Branch(-PT/BR)]\n * [http://scp-cs.wikidot.com SCP Nadace(-CZ)]\n* Background\n * [[[about-the-scp-foundation|About the Foundation]]]\n * [[[Object Classes]]]\n * [[[personnel-and-character-dossier|Personnel Dossier]]]\n * [[[security-clearance-levels|Security & Clearance]]]\n * [[[Secure Facilities Locations|Secure Facilities]]]\n * [[[Task Forces]]]\n* Info Pages\n * [[[Guide Hub]]]\n * [[[usertools|User Tools]]]\n * [[[Tag Search]]]\n * [[[Meet The Staff]]]\n * [[[criticism-policy|Criticism Policy]]]\n * [[[Licensing Guide]]]\n * [[[Image Use Policy]]]\n * [[[chat-guide|Chat Guidelines]]]\n * [[[deletions-guide|Deletions Guidelines]]]\n * [[[seminars-hub|Seminars and Workshops]]]\n * [[[Donations Policy]]]\n * [[[links|Links]]]\n[[/div]]\n\n[[div class="mobile-top-bar"]]\n\n[[div class="open-menu"]]\n[#side-bar ≡]\n[[/div]]\n\n\n* SCPs\n * [[[scp-series-6|Series VI]]]\n * [[[scp-series-5|Series V]]]\n * [[[scp-series-5-tales-edition|» Series V Tales]]]\n * [[[scp-series-4|Series IV]]]\n * [[[scp-series-4-tales-edition|» Series IV Tales]]]\n * [[[scp-series-3|Series III]]]\n * [[[scp-series-3-tales-edition|» Series III Tales]]]\n * [[[scp-series-2|Series II]]]\n * [[[scp-series-2-tales-edition|» Series II Tales]]]\n * [[[scp-series|Series I]]]\n * [[[scp-series-1-tales-edition|» Series I Tales]]]\n* Tales\n * [[[Foundation Tales]]]\n * [[[Series Archive]]]\n * [[[incident-reports-eye-witness-interviews-and-personal-logs|Incident Reports]]]\n * [[[Creepy-Pasta|CreepyPasta Archive]]]\n* Library\n * [[[User Curated Lists]]]\n * [[[Joke SCPs]]]\n * [[[joke-scps-tales-edition|» Joke SCPs Tales]]]\n * [[[scp-ex|Explained SCPs]]]\n * [[[explained-scps-tales-edition|» Explained SCPs Tales]]]\n * [[[GoI Formats]]]\n * [[[Audio Adaptations]]]\n * [[[scp-artwork-hub|Artwork Hub]]]\n * [[[Contest Archive]]]\n* Universe\n * [[[Canon Hub]]]\n * [[[groups-of-interest|GoIs]]]\n * [[[log-of-anomalous-items|Anomalous Items]]]\n * [[[log-of-extranormal-events|Extranormal Events]]]\n * [[[log-of-unexplained-locations|Unexplained Locations]]]\n * [[[Object Classes]]]\n * [[[personnel-and-character-dossier|Personnel Dossier]]]\n * [[[security-clearance-levels|Security & Clearance]]]\n * [[[Secure Facilities Locations|Secure Facilities]]]\n * [[[Task Forces]]]\n* Guides\n * [[[Guide Hub]]]\n * [[[Guide for Newbies]]]\n * [[[How to Write an SCP]]]\n * [[[Image Use Policy]]]\n * [[[chat-guide|Chat Guidelines]]]\n * [[[FAQ]]]\n * [[[Site Rules]]]\n * [[[deletions-guide|Deletions Guidelines]]]\n * [[[criticism-policy|Criticism Policy]]]\n * [[[seminars-hub|Seminars and Workshops]]]\n * [[[links|Links]]]\n* [# Admin]\n * [[[admin:manage|Site Manager]]]\n\n[!-- top nav menu, use only one bulleted list above --]\n[[/div]]
169	[[div class="top-bar"]]\n* SCP Series\n * [[[scp-series-6|Series VI]]]\n * [[[scp-series-5|Series V]]]\n * [[[scp-series-5-tales-edition|» Series V Tales]]]\n * [[[scp-series-4|Series IV]]]\n * [[[scp-series-4-tales-edition|» Series IV Tales]]]\n * [[[scp-series-3|Series III]]]\n * [[[scp-series-3-tales-edition|» Series III Tales]]]\n * [[[scp-series-2|Series II]]]\n * [[[scp-series-2-tales-edition|» Series II Tales]]]\n * [[[scp-series|Series I]]]\n * [[[scp-series-1-tales-edition|» Series I Tales]]]\n* Tales\n * [[[Foundation Tales]]]\n * [[[Series Archive]]]\n * [[[incident-reports-eye-witness-interviews-and-personal-logs|Incident Reports]]]\n * [[[Creepy-Pasta|CreepyPasta Archive]]]\n* Library\n * [[[user-curated-lists|User-Curated Lists]]]\n * [[[Joke SCPs]]]\n * [[[joke-scps-tales-edition|» Joke SCPs Tales]]]\n * [[[scp-ex|Explained SCPs]]]\n * [[[explained-scps-tales-edition|» Explained SCPs Tales]]]\n * [[[GoI Formats]]]\n * [[[Audio Adaptations]]]\n * [[[SCP Artwork Hub]]]\n * [[[Contest Archive]]]\n* Universe\n * [[[Canon Hub|Canons]]]\n * [[[Groups of Interest]]]\n * [[[log-of-anomalous-items|Anomalous Items]]]\n * [[[log-of-extranormal-events|Extranormal Events]]]\n * [[[log-of-unexplained-locations|Unexplained Locations]]]\n* SCP Global\n * [http://scp-int.wikidot.com International Translation Archive]\n * [http://scp-ru.wikidot.com Russian Branch(-RU)]\n * [http://ko.scp-wiki.net/ SCP 재단(-KO)]\n * [http://scp-wiki-cn.wikidot.com SCP基金会(-CN)]\n * [http://fondationscp.wikidot.com Fondation SCP(-FR)]\n * [http://scp-wiki.net.pl SCP Polska Filia(-PL)]\n * [http://lafundacionscp.wikidot.com La Fundación SCP(-ES)]\n * [http://scp-th.wikidot.com สถาบัน SCP(-TH)]\n * [http://scp-jp.wikidot.com SCP財団(-JP)]\n * [http://scp-wiki-de.wikidot.com SCP Deutschland(-DE)]\n * [http://fondazionescp.wikidot.com Fondazione SCP(-IT)]\n * [http://scp-ukrainian.wikidot.com Ukrainian Branch(-UA)]\n * [http://scp-pt-br.wikidot.com/ Lusófona Branch(-PT/BR)]\n * [http://scp-cs.wikidot.com SCP Nadace(-CZ)]\n* Background\n * [[[about-the-scp-foundation|About the Foundation]]]\n * [[[Object Classes]]]\n * [[[personnel-and-character-dossier|Personnel Dossier]]]\n * [[[security-clearance-levels|Security & Clearance]]]\n * [[[Secure Facilities Locations|Secure Facilities]]]\n * [[[Task Forces]]]\n* Info Pages\n * [[[Guide Hub]]]\n * [[[usertools|User Tools]]]\n * [[[Tag Search]]]\n * [[[Meet The Staff]]]\n * [[[criticism-policy|Criticism Policy]]]\n * [[[Licensing Guide]]]\n * [[[Image Use Policy]]]\n * [[[chat-guide|Chat Guidelines]]]\n * [[[deletions-guide|Deletions Guidelines]]]\n * [[[seminars-hub|Seminars and Workshops]]]\n * [[[Donations Policy]]]\n * [[[links|Links]]]\n[[/div]]\n\n[[div class="mobile-top-bar"]]\n\n[[div class="open-menu"]]\n[#side-bar ≡]\n[[/div]]\n\n\n* SCPs\n * [[[scp-series-6|Series VI]]]\n * [[[scp-series-5|Series V]]]\n * [[[scp-series-5-tales-edition|» Series V Tales]]]\n * [[[scp-series-4|Series IV]]]\n * [[[scp-series-4-tales-edition|» Series IV Tales]]]\n * [[[scp-series-3|Series III]]]\n * [[[scp-series-3-tales-edition|» Series III Tales]]]\n * [[[scp-series-2|Series II]]]\n * [[[scp-series-2-tales-edition|» Series II Tales]]]\n * [[[scp-series|Series I]]]\n * [[[scp-series-1-tales-edition|» Series I Tales]]]\n* Tales\n * [[[Foundation Tales]]]\n * [[[Series Archive]]]\n * [[[incident-reports-eye-witness-interviews-and-personal-logs|Incident Reports]]]\n * [[[Creepy-Pasta|CreepyPasta Archive]]]\n* Library\n * [[[User Curated Lists]]]\n * [[[Joke SCPs]]]\n * [[[joke-scps-tales-edition|» Joke SCPs Tales]]]\n * [[[scp-ex|Explained SCPs]]]\n * [[[explained-scps-tales-edition|» Explained SCPs Tales]]]\n * [[[GoI Formats]]]\n * [[[Audio Adaptations]]]\n * [[[scp-artwork-hub|Artwork Hub]]]\n * [[[Contest Archive]]]\n* Universe\n * [[[Canon Hub]]]\n * [[[groups-of-interest|GoIs]]]\n * [[[log-of-anomalous-items|Anomalous Items]]]\n * [[[log-of-extranormal-events|Extranormal Events]]]\n * [[[log-of-unexplained-locations|Unexplained Locations]]]\n * [[[Object Classes]]]\n * [[[personnel-and-character-dossier|Personnel Dossier]]]\n * [[[security-clearance-levels|Security & Clearance]]]\n * [[[Secure Facilities Locations|Secure Facilities]]]\n * [[[Task Forces]]]\n* Guides\n * [[[Guide Hub]]]\n * [[[Guide for Newbies]]]\n * [[[How to Write an SCP]]]\n * [[[Image Use Policy]]]\n * [[[chat-guide|Chat Guidelines]]]\n * [[[FAQ]]]\n * [[[Site Rules]]]\n * [[[deletions-guide|Deletions Guidelines]]]\n * [[[criticism-policy|Criticism Policy]]]\n * [[[seminars-hub|Seminars and Workshops]]]\n * [[[links|Links]]]\n*  Admin\n * [[[admin:manage|Site Manager]]]\n\n[!-- top nav menu, use only one bulleted list above --]\n[[/div]]
170	[[div class="top-bar"]]\n* SCP Series\n * [[[scp-series-6|Series VI]]]\n * [[[scp-series-5|Series V]]]\n * [[[scp-series-5-tales-edition|» Series V Tales]]]\n * [[[scp-series-4|Series IV]]]\n * [[[scp-series-4-tales-edition|» Series IV Tales]]]\n * [[[scp-series-3|Series III]]]\n * [[[scp-series-3-tales-edition|» Series III Tales]]]\n * [[[scp-series-2|Series II]]]\n * [[[scp-series-2-tales-edition|» Series II Tales]]]\n * [[[scp-series|Series I]]]\n * [[[scp-series-1-tales-edition|» Series I Tales]]]\n* Tales\n * [[[Foundation Tales]]]\n * [[[Series Archive]]]\n * [[[incident-reports-eye-witness-interviews-and-personal-logs|Incident Reports]]]\n * [[[Creepy-Pasta|CreepyPasta Archive]]]\n* Library\n * [[[user-curated-lists|User-Curated Lists]]]\n * [[[Joke SCPs]]]\n * [[[joke-scps-tales-edition|» Joke SCPs Tales]]]\n * [[[scp-ex|Explained SCPs]]]\n * [[[explained-scps-tales-edition|» Explained SCPs Tales]]]\n * [[[GoI Formats]]]\n * [[[Audio Adaptations]]]\n * [[[SCP Artwork Hub]]]\n * [[[Contest Archive]]]\n* Universe\n * [[[Canon Hub|Canons]]]\n * [[[Groups of Interest]]]\n * [[[log-of-anomalous-items|Anomalous Items]]]\n * [[[log-of-extranormal-events|Extranormal Events]]]\n * [[[log-of-unexplained-locations|Unexplained Locations]]]\n* SCP Global\n * [http://scp-int.wikidot.com International Translation Archive]\n * [http://scp-ru.wikidot.com Russian Branch(-RU)]\n * [http://ko.scp-wiki.net/ SCP 재단(-KO)]\n * [http://scp-wiki-cn.wikidot.com SCP基金会(-CN)]\n * [http://fondationscp.wikidot.com Fondation SCP(-FR)]\n * [http://scp-wiki.net.pl SCP Polska Filia(-PL)]\n * [http://lafundacionscp.wikidot.com La Fundación SCP(-ES)]\n * [http://scp-th.wikidot.com สถาบัน SCP(-TH)]\n * [http://scp-jp.wikidot.com SCP財団(-JP)]\n * [http://scp-wiki-de.wikidot.com SCP Deutschland(-DE)]\n * [http://fondazionescp.wikidot.com Fondazione SCP(-IT)]\n * [http://scp-ukrainian.wikidot.com Ukrainian Branch(-UA)]\n * [http://scp-pt-br.wikidot.com/ Lusófona Branch(-PT/BR)]\n * [http://scp-cs.wikidot.com SCP Nadace(-CZ)]\n* Background\n * [[[about-the-scp-foundation|About the Foundation]]]\n * [[[Object Classes]]]\n * [[[personnel-and-character-dossier|Personnel Dossier]]]\n * [[[security-clearance-levels|Security & Clearance]]]\n * [[[Secure Facilities Locations|Secure Facilities]]]\n * [[[Task Forces]]]\n* Info Pages\n * [[[Guide Hub]]]\n * [[[usertools|User Tools]]]\n * [[[Tag Search]]]\n * [[[Meet The Staff]]]\n * [[[criticism-policy|Criticism Policy]]]\n * [[[Licensing Guide]]]\n * [[[Image Use Policy]]]\n * [[[chat-guide|Chat Guidelines]]]\n * [[[deletions-guide|Deletions Guidelines]]]\n * [[[seminars-hub|Seminars and Workshops]]]\n * [[[Donations Policy]]]\n * [[[links|Links]]]\n*  Admin\n * [[[admin:manage|Site Manager]]]\n[[/div]]\n\n[[div class="mobile-top-bar"]]\n\n[[div class="open-menu"]]\n[#side-bar ≡]\n[[/div]]\n\n\n* SCPs\n * [[[scp-series-6|Series VI]]]\n * [[[scp-series-5|Series V]]]\n * [[[scp-series-5-tales-edition|» Series V Tales]]]\n * [[[scp-series-4|Series IV]]]\n * [[[scp-series-4-tales-edition|» Series IV Tales]]]\n * [[[scp-series-3|Series III]]]\n * [[[scp-series-3-tales-edition|» Series III Tales]]]\n * [[[scp-series-2|Series II]]]\n * [[[scp-series-2-tales-edition|» Series II Tales]]]\n * [[[scp-series|Series I]]]\n * [[[scp-series-1-tales-edition|» Series I Tales]]]\n* Tales\n * [[[Foundation Tales]]]\n * [[[Series Archive]]]\n * [[[incident-reports-eye-witness-interviews-and-personal-logs|Incident Reports]]]\n * [[[Creepy-Pasta|CreepyPasta Archive]]]\n* Library\n * [[[User Curated Lists]]]\n * [[[Joke SCPs]]]\n * [[[joke-scps-tales-edition|» Joke SCPs Tales]]]\n * [[[scp-ex|Explained SCPs]]]\n * [[[explained-scps-tales-edition|» Explained SCPs Tales]]]\n * [[[GoI Formats]]]\n * [[[Audio Adaptations]]]\n * [[[scp-artwork-hub|Artwork Hub]]]\n * [[[Contest Archive]]]\n* Universe\n * [[[Canon Hub]]]\n * [[[groups-of-interest|GoIs]]]\n * [[[log-of-anomalous-items|Anomalous Items]]]\n * [[[log-of-extranormal-events|Extranormal Events]]]\n * [[[log-of-unexplained-locations|Unexplained Locations]]]\n * [[[Object Classes]]]\n * [[[personnel-and-character-dossier|Personnel Dossier]]]\n * [[[security-clearance-levels|Security & Clearance]]]\n * [[[Secure Facilities Locations|Secure Facilities]]]\n * [[[Task Forces]]]\n* Guides\n * [[[Guide Hub]]]\n * [[[Guide for Newbies]]]\n * [[[How to Write an SCP]]]\n * [[[Image Use Policy]]]\n * [[[chat-guide|Chat Guidelines]]]\n * [[[FAQ]]]\n * [[[Site Rules]]]\n * [[[deletions-guide|Deletions Guidelines]]]\n * [[[criticism-policy|Criticism Policy]]]\n * [[[seminars-hub|Seminars and Workshops]]]\n * [[[links|Links]]]\n*  Admin\n * [[[admin:manage|Site Manager]]]\n\n[!-- top nav menu, use only one bulleted list above --]\n[[/div]]
186	According to [http://en.wikipedia.org/wiki/Wiki Wikipedia], the world largest wiki site:\n\n> A //Wiki// ([ˈwiː.kiː] <wee-kee> or [ˈwɪ.kiː] <wick-ey>) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.\n\nAnd that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.
281	++ Top Sites\n[[module MostActiveSites]]\n\n++ Recent edits (all websites)\n[[module RecentWRevisions]]\n\n++ Top Forums\n[[module MostActiveForums]]\n\n++ Top Sites\n[[module SiteGrid limit="20"]]\n\n++ New users\n[[module NewWUsers limit="50"]]
287	[[module ForumStart]]\n[!-- please do not alter this page if you want to keep your forum working --]
171	[[div class="side-block" style="background-color: #e5e5ff;"]]\n[[div class="menu-item"]]\n[[=]]\n[[image icon-DeviantArt-2020.png link="http://scp-foundation.deviantart.com/" style="width:30px; height:30px;"  alt="SCP DeviantArt"]][[image icon-Facebook-2020.png link="https://www.facebook.com/scpfoundation" style="width:30px; height:30px;" alt="Facebook"]][[image icon-Twitter-2020.png link="https://twitter.com/scpwiki" style="width:30px; height:30px;" alt="Twitter"]][[image icon-Reddit-2020.png link="http://www.reddit.com/r/SCP" style="width:30px; height:30px;" alt="Reddit"]][[image icon-Tumblr-2020.png link="http://scp-wiki-official.tumblr.com/" style="width:30px; height:30px;" alt="Tumblr"]][[image icon-Instagram-2020.png link="https://www.instagram.com/scpfoundationwiki/" style="width:30px; height:30px;" alt="Instagram"]]\n[[/=]]\n[[/div]]\n[[/div]]\n\n~~~~\n\n[[div class="side-block"]]\n\n\n\n[[div class="menu-item"]]\n[[image home.png]][/ Main]\n[[/div]]\n[[div class="heading"]]\nSCP by Series\n[[/div]]\n[[div class="menu-item"]]\n[[image series.png]] [[[scp-series|I]]] | [[[scp-series-2|II]]] | [[[scp-series-3|III]]] | [[[scp-series-4|IV]]] | [[[scp-series-5|V]]] | [[[scp-series-6|VI]]]  \n[[/div]]\n[[div class="heading"]]\nSCP Tales by Series\n[[/div]]\n[[div class="menu-item"]]\n[[image series.png]] [[[scp-series-1-tales-edition|I]]] | [[[scp-series-2-tales-edition|II]]] | [[[scp-series-3-tales-edition|III]]] | [[[scp-series-4-tales-edition|IV]]] | [[[scp-series-5-tales-edition|V]]]\n[[/div]]\n\n[[div class="heading"]]\nSCP Library\n[[/div]]\n[[div class="menu-item"]]\n[[image series.png]][[[foundation-tales|Tales]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image series.png]][[[canon-hub|Canons]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image series.png]][[[scp-international|International SCP Hub]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image series.png]][[[GoI Formats]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image series.png]][[[scp-ex|Explained SCPs]]]\n[[/div]]\n\n[[div class="heading"]]\nDiscover Content\n[[/div]]\n[[div class="menu-item"]]\n[[image default.png]][[[Top Rated Pages This Month| Top Rated New Pages]]]\n[[/div]]\n[[div class="menu-item sub-item"]]\n[[image default.png]][[[new-pages-feed| Newly Created Pages]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image main.png]][[[random:random-scp|Random SCP]]] | [[[random:random-tale|Tale]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image main.png]][[[system:recent-changes| Recent Changes]]] | [[[http://www.scp-wiki.net/most-recently-edited| Edits]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image default.png]][[[Lowest Rated Pages]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image main.png]][[[guide-hub|Guides & Essays]]]\n[[/div]]\n[[div class="menu-item sub-item"]]\n[[image default.png]][[[Contribute]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image main.png]][[[http://www.scp-wiki.net/young-and-under-30|Underread & Underrated]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image main.png]][[[seminars-hub|Seminars & Workshops]]]\n[[/div]]\n\n[[div class="heading"]]\nSCP Community\n[[/div]]\n[[div class="menu-item"]]\n[[image help.png]][[[Site Rules]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image main.png]][[[system:join|Join the Site!]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image forum.png]][[[forum:start|Forum]]] | [[[forum:recent-posts|New Posts]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image forum.png]][[[chat-guide|Chat With Us!]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image default.png]][[[authors-pages|Authors' Pages]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image main.png]][[[news|Site News Hub]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image default.png]][[[http://05command.wikidot.com/staff-policy-hub|Policy Hub]]]\n[[/div]]\n[[/div]]\n\n~~~~\n\n\n[[div class="side-block" style="background-color: #fff0f0;"]]\n[[div class="heading"]]\nUser Resources\n[[/div]]\n[[div class="menu-item"]]\n[[image help.png]][[[How to Write an SCP]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image default.png]][[[Tag Search]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image default.png]][[[usertools|User Tools]]]\n[[/div]]\n[[div class="menu-item"]]\n[[image default.png]][http://www.wikidot.com/doc:start Wiki Syntax][[/div]]\n[[div class="menu-item"]]\n[[image default.png]][[[Sandbox]]][[/div]]\n[[div class="menu-item"]]\n[[image main.png]][http://05command.wikidot.com Staff Site][[/div]]\n[[div class="menu-item"]]\n[[image help.png]][[[Contact Staff]]]\n[[/div]]\n[[/div]]\n\n\n~~~~\n\n[[a href="#" class="close-menu"]]\n[[image black.png style="z-index:-1; opacity: 0.3;"]]\n[[/a]]\n\n[[div class="scpnet-interwiki-wrapper"]]\n[[module ListPages range="." limit="1"]]\n      [[iframe http://interwiki.scpdb.org/?wiki=scp-wiki&lang=en&page=%%category%%:%%name%% class="scpnet-interwiki-frame"]]\n[[/module]]\n[[/div]]
172	[[div class="top-bar"]]\n* SCP Series\n * [[[scp-series-6|Series VI]]]\n * [[[scp-series-5|Series V]]]\n * [[[scp-series-5-tales-edition|» Series V Tales]]]\n * [[[scp-series-4|Series IV]]]\n * [[[scp-series-4-tales-edition|» Series IV Tales]]]\n * [[[scp-series-3|Series III]]]\n * [[[scp-series-3-tales-edition|» Series III Tales]]]\n * [[[scp-series-2|Series II]]]\n * [[[scp-series-2-tales-edition|» Series II Tales]]]\n * [[[scp-series|Series I]]]\n * [[[scp-series-1-tales-edition|» Series I Tales]]]\n* Tales\n * [[[Foundation Tales]]]\n * [[[Series Archive]]]\n * [[[incident-reports-eye-witness-interviews-and-personal-logs|Incident Reports]]]\n * [[[Creepy-Pasta|CreepyPasta Archive]]]\n* Library\n * [[[user-curated-lists|User-Curated Lists]]]\n * [[[Joke SCPs]]]\n * [[[joke-scps-tales-edition|» Joke SCPs Tales]]]\n * [[[scp-ex|Explained SCPs]]]\n * [[[explained-scps-tales-edition|» Explained SCPs Tales]]]\n * [[[GoI Formats]]]\n * [[[Audio Adaptations]]]\n * [[[SCP Artwork Hub]]]\n * [[[Contest Archive]]]\n* Universe\n * [[[Canon Hub|Canons]]]\n * [[[Groups of Interest]]]\n * [[[log-of-anomalous-items|Anomalous Items]]]\n * [[[log-of-extranormal-events|Extranormal Events]]]\n * [[[log-of-unexplained-locations|Unexplained Locations]]]\n* SCP Global\n * [http://scp-int.wikidot.com International Translation Archive]\n * [http://scp-ru.wikidot.com Russian Branch(-RU)]\n * [http://ko.scp-wiki.net/ SCP 재단(-KO)]\n * [http://scp-wiki-cn.wikidot.com SCP基金会(-CN)]\n * [http://fondationscp.wikidot.com Fondation SCP(-FR)]\n * [http://scp-wiki.net.pl SCP Polska Filia(-PL)]\n * [http://lafundacionscp.wikidot.com La Fundación SCP(-ES)]\n * [http://scp-th.wikidot.com สถาบัน SCP(-TH)]\n * [http://scp-jp.wikidot.com SCP財団(-JP)]\n * [http://scp-wiki-de.wikidot.com SCP Deutschland(-DE)]\n * [http://fondazionescp.wikidot.com Fondazione SCP(-IT)]\n * [http://scp-ukrainian.wikidot.com Ukrainian Branch(-UA)]\n * [http://scp-pt-br.wikidot.com/ Lusófona Branch(-PT/BR)]\n * [http://scp-cs.wikidot.com SCP Nadace(-CZ)]\n* Background\n * [[[about-the-scp-foundation|About the Foundation]]]\n * [[[Object Classes]]]\n * [[[personnel-and-character-dossier|Personnel Dossier]]]\n * [[[security-clearance-levels|Security & Clearance]]]\n * [[[Secure Facilities Locations|Secure Facilities]]]\n * [[[Task Forces]]]\n* Info Pages\n * [[[Guide Hub]]]\n * [[[usertools|User Tools]]]\n * [[[Tag Search]]]\n * [[[Meet The Staff]]]\n * [[[criticism-policy|Criticism Policy]]]\n * [[[Licensing Guide]]]\n * [[[Image Use Policy]]]\n * [[[chat-guide|Chat Guidelines]]]\n * [[[deletions-guide|Deletions Guidelines]]]\n * [[[seminars-hub|Seminars and Workshops]]]\n * [[[Donations Policy]]]\n * [[[links|Links]]]\n*  Admin\n * [[[admin:manage|Site Manager]]]\n[[/div]]\n\n[[div class="mobile-top-bar"]]\n\n[[div class="open-menu"]]\n[#side-bar ≡]\n[[/div]]\n\n\n* SCPs\n * [[[scp-series-6|Series VI]]]\n * [[[scp-series-5|Series V]]]\n * [[[scp-series-5-tales-edition|» Series V Tales]]]\n * [[[scp-series-4|Series IV]]]\n * [[[scp-series-4-tales-edition|» Series IV Tales]]]\n * [[[scp-series-3|Series III]]]\n * [[[scp-series-3-tales-edition|» Series III Tales]]]\n * [[[scp-series-2|Series II]]]\n * [[[scp-series-2-tales-edition|» Series II Tales]]]\n * [[[scp-series|Series I]]]\n * [[[scp-series-1-tales-edition|» Series I Tales]]]\n* Tales\n * [[[Foundation Tales]]]\n * [[[Series Archive]]]\n * [[[incident-reports-eye-witness-interviews-and-personal-logs|Incident Reports]]]\n * [[[Creepy-Pasta|CreepyPasta Archive]]]\n* Library\n * [[[User Curated Lists]]]\n * [[[Joke SCPs]]]\n * [[[joke-scps-tales-edition|» Joke SCPs Tales]]]\n * [[[scp-ex|Explained SCPs]]]\n * [[[explained-scps-tales-edition|» Explained SCPs Tales]]]\n * [[[GoI Formats]]]\n * [[[Audio Adaptations]]]\n * [[[scp-artwork-hub|Artwork Hub]]]\n * [[[Contest Archive]]]\n* Universe\n * [[[Canon Hub]]]\n * [[[groups-of-interest|GoIs]]]\n * [[[log-of-anomalous-items|Anomalous Items]]]\n * [[[log-of-extranormal-events|Extranormal Events]]]\n * [[[log-of-unexplained-locations|Unexplained Locations]]]\n * [[[Object Classes]]]\n * [[[personnel-and-character-dossier|Personnel Dossier]]]\n * [[[security-clearance-levels|Security & Clearance]]]\n * [[[Secure Facilities Locations|Secure Facilities]]]\n * [[[Task Forces]]]\n* Guides\n * [[[Guide Hub]]]\n * [[[Guide for Newbies]]]\n * [[[How to Write an SCP]]]\n * [[[Image Use Policy]]]\n * [[[chat-guide|Chat Guidelines]]]\n * [[[FAQ]]]\n * [[[Site Rules]]]\n * [[[deletions-guide|Deletions Guidelines]]]\n * [[[criticism-policy|Criticism Policy]]]\n * [[[seminars-hub|Seminars and Workshops]]]\n * [[[links|Links]]]\n*  Admin\n * [[[admin:manage|Site Manager]]]\n\n[!-- top nav menu, use only one bulleted list above --]\n[[/div]]
177	[[module css]]\n#page-title { display: none; }\n.feature-block {\n    height: 230px;\n}\n.feature-block .panel-body {\n    height: 160px;\n    overflow-y: hidden;\n}\n.feature-block .feature-title {\n    font-size: 120%;\n    font-weight: bold;\n}\n.feature-block .feature-subtitle {\n    font-size: 90%;\n}\n\n.feature-block .feature-title > p,\n.feature-block .feature-subtitle > p {\n    margin: 0;\n}\n\n.news-block .panel-body {\n    padding: 0;\n}\n.news-block .panel-body .alternate {\n    padding: 5px 10px;\n}\n}\n.news-block .news-title {\n    font-weight: bold;\n    font-size: 110%;\n    margin-bottom: 5px;\n    color: #633;\n}\n.news-block .news-content {\n    margin: 5px 0;\n}\n.news-block .news-title > p,\n.news-block .news-content > p {\n    margin: 0;\n}\n\n.international-block .panel-body {\n    padding: 0;\n}\n.international-entry {\n    padding: 5px 10px;\n}\n.international-entry:nth-child(even),\n.alternate:nth-child(even) {\n    background-color: rgba(255,255,255,.9);\n    border-top: solid 1px #eeeee0;\n    border-bottom: solid 1px #eeeee0;\n}\n.international-entry div > p {\n    margin: 0;\n}\n.international-entry .international-title {\n    font-weight: bold;\n    font-size: 110%;\n}\n.international-entry .international-title img {\n    width: 16px;\n    height: 11px;\n    box-shadow: 0 1px 3px rgba(0,0,0,.5);\n}\n.international-entry .international-description {\n    font-size: 90%;\n    font-style: italic;\n    margin: 5px 20px;\n}\n.international-entry .international-footer {\n    font-size: #333;\n    font-size: 80%;\n    margin: 5px 0;\n}\n\n.content-panel p { padding: 0 0.75em; }\n[[/module]]\n\n[[=]]\n[!-- This image acts as a thumbnail. --]\n[[image http://scp-wiki.wdfiles.com/local--files/main/250_logo.png width="0" height="0"]]\n[[/=]]\n\n[[div style="text-align: center; color: #600;"]]\n[[div class="unmargined"]]\nWARNING: THE FOUNDATION DATABASE IS\n[[/div]]\n[[div class="unmargined" style="font-size: 400%; font-weight: bold; margin: 2px 0 5px;"]]\nCLASSIFIED\n[[/div]]\n[[div class="unmargined"]]\nACCESS BY UNAUTHORIZED PERSONNEL IS STRICTLY PROHIBITED\nPERPETRATORS WILL BE TRACKED, LOCATED, AND DETAINED\n[[/div]]\n[[/div]]\n\n[!-- * This is commented out until it's needed. * --]\n[[div class="content-panel centered standalone"]]\n\n\n[[/div]]\n[!-- --]\n\n[!-- * This is commented out until it's needed. * --]\n\n\n[!-- --]\n\n[!-- * This is commented out until it's needed. * --]\n\n\n[!-- [[div class="content-panel centered standalone"]]\n[[size 300%]] placeholder text\n[[/size]]\n[[size 150%]]\nplaceholder text\n[[/size]]\n\n\n[[/div]] --]\n[[div class="content-panel centered standalone"]]\n++ [[[http://scp-wiki.wikidot.com/exquisite-corpse-contest |Exquisite Corpse Contest]]]\n\n[[div class="scp-image-block block-right" style="width:200px;"]]\n\n[[image http://scp-wiki.wdfiles.com/local--files/exquisite-corpse-contest/exquisite-corpse-logo.png width="200px" style="width:200px;" link="http://scp-wiki.wikidot.com/exquisite-corpse-contest"]]\n[[/div]]\n\nPhase Two of The Exquisite Corpse Contest has begun! All participants have received their containment procedures and are free to begin posting their articles! Write an SCP in an exceedingly unorthodox fashion: as an [https://en.wikipedia.org/wiki/Exquisite_corpse exquisite corpse]! It's a team contest without a team!\n\n+ >> [[[http://scp-wiki.wikidot.com/exquisite-corpse-contest |Contest Page]]]\n\n\n[[/div]]\n\n[[div class="content-panel centered standalone"]]\n**Next [[[seminars-hub | Seminar/Workshop]]]: Welcome to the Wiki: SCP Foundation Orientation - June 13, 2020 @ 2 pm EDT**\n[[include :scp-wiki:component:tz\n| time=14:00\n| tz=EDT\n]]\n[[/div]]\n\n\n\n\n\n[[div class="feature-block"]]\n[[div class="content-panel left-column"]]\n[[div class="panel-heading"]]\nFeatured SCP Article\n[[/div]]\n[[div class="panel-body"]]\n[[div class="feature-title"]]\n[[[SCP-5058 | SCP-5058: Our Normal Fellow Humans]]]\n[[/div]]\n[[div class="feature-subtitle"]]\nby [[user AlanDaris]]\n[[/div]]\n\n//SCP-5058 are entities that visually resemble Homo sapiens (humans)...//\n\n(Featured by gee0765 and AlanDaris)\n\n[[/div]]\n[[div class="panel-footer"]]\n[[[Featured SCP Archive II | Featured SCP Archive]]]\n[[/div]]\n[[/div]]\n\n[[div class="content-panel right-column"]]\n[[div class="panel-heading"]]\nFeatured Tale\n[[/div]]\n[[div class="panel-body"]]\n[[div class="feature-title"]]\n[[[siobhra | Síobhra]]]\n[[/div]]\n[[div class="feature-subtitle"]]\nby [[user Ihp]]\n[[/div]]\n\n//They've come back.//\n\n(Featured by MalyceGraves and OCuin)\n\n[[/div]]\n[[div class="panel-footer"]]\n[[[featured-tale-archive-ii | Featured Tale Archive]]]\n[[/div]]\n[[/div]]\n\n[[div class="content-panel left-column"]]\n[[div class="panel-heading"]]\nFeatured GoI Format document\n[[/div]]\n[[div class="panel-body"]]\n[[div class="feature-title"]]\n[[[critter-profile-zargoth | Critter Profile: ZARGOTH, DESTROYER OF DIMENSIONS!]]]\n[[/div]]\n[[div class="feature-subtitle"]]\nby [[user DarkStuff]] and [[user Ellie3]]\n[[/div]]\n\n//IMPORTANT: This Critter Profile is HORRIFICALLY GRUESOME!//\n\n(Featured by TheMightyMcB and Naveil)\n\n[[/div]]\n[[div class="panel-footer"]]\n[[[Featured GoI Format Archive]]]\n[[/div]]\n[[/div]]\n\n[[div class="content-panel right-column"]]\n[[div class="panel-heading"]]\nReviewers' Spotlight: //sirpudding//\n[[/div]]\n[[div class="panel-body"]]\n[[div class="feature-title"]]\n[[[SCP-2656 | SCP-2656: The Idiot Box]]]\n[[/div]]\n[[div class="feature-subtitle"]]\nby [[user CupertinoEffect]]\n[[/div]]\n\n//Stage 4: Sections of the brain are further dissected.//\n\n[[/div]]\n[[div class="panel-footer"]]\n[[[Reviewers Spotlight Archive | Reviewers' Spotlight Archive]]]\n[[/div]]\n[[/div]]\n[[/div]]\n\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n[[div class="news-block content-panel"]]\n[[div class="panel-heading"]]\nNews\n[[/div]]\n[[div class="panel-body"]]\n\n[!-- PLEASE LIMIT THE PAGE TO FIVE NEWS ARTICLES --]\n[!-- Pretty sure the five-page limit was specifically for the old template. -Quik --]\n[!-- Troy was here. Moose is deadly. Zyn does not approve of calling others ugly. Esko is a bear. Light's just radiant. Roget is the actual worst. Drew is the Bear Overlord. A Kaktus is a desert plant. Lurkd is hiding under your chair. Decibelle is a princess~ Silber is better than bronze. ARD is bizarre. Tuomey could use a drink.--]\n[!-- How much wood could a woodchuck chuck with an automatic wood chucker? -Taylor --]\n\n[[div class="alternate unmargined"]]\n[[div class="news-title"]]\nMay 1, 2020\n[[/div]]\n[[div class="news-content"]]\nThe SCP Wiki's new site contest is open for entering - go check out the [[[Exquisite Corpse Contest]]]!\n\n[[/div]]\n[[/div]]\n\n[[div class="alternate unmargined"]]\n[[div class="news-title"]]\nApril 19, 2020\n[[/div]]\n[[div class="news-content"]]\nThe Wanderer's Library's new site contest is open for posting - go check out the [http://wanderers-library.wikidot.com/wanderers-depths-contest Wanderer's Depths]!\n\n[[/div]]\n[[/div]]\n\n[[div class="alternate unmargined"]]\n[[div class="news-title"]]\nApril 1, 2020\n[[/div]]\n[[div class="news-content"]]\nA number of articles changed their seasonal colors for April Fool's Day and turned into [[[april-fools-2020|Super Cool Plants]]] entries.\n\n[[/div]]\n[[/div]]\n\n[[div class="alternate unmargined"]]\n[[div class="news-title"]]\nMarch 29, 2020\n[[/div]]\n[[div class="news-content"]]\n[[[144-hour-jam-contest-two|]]] has come to a close! Nearly two hundred contest entries were submitted over the course of six days!\n\n[[/div]]\n[[/div]]\n\n[[div class="alternate unmargined"]]\n[[div class="news-title"]]\nFebruary 8, 2020\n[[/div]]\n[[div class="news-content"]]\nThe [[[SCP-5000]]] Contest is over! Congratulations to [[*user Tanhony]] for winning the SCP-5000 slot!\n\n[[/div]]\n[[/div]]\n\n\n[[/div]]\n\n[[div class="panel-footer"]]\n\n[[[Archived News]]]\n\n[[/div]]\n\n[[/div]]\n\n\n----\n\n[[div class="international-block content-panel"]]\n[[div class="panel-heading"]]\nOfficial Sister Sites\n[[/div]]\n[[div class="panel-body"]]\n\n[[div class="international-entry"]]\n[[div class="international-title"]]\n[[image SerpHand.png]] [http://wanderers-library.wikidot.com The Wanderers Library] -- Fiction From Another World.\n[[/div]]\n[[div class="international-description"]]\nThe Wanderers Library is the Foundation's sister site and features the endless stories contained within the Library, home to the Serpents Hand and readers of all shapes and sizes.\n[[/div]]\n[[div class="international-footer"]]\nAdmins: -- [[user Roget]] -- [[user Rounderhouse]] -- [[user Rumetzen]]\n[[/div]]\n[[/div]]\n\n[[/div]]\n[[div class="panel-footer"]]\n  [!-- Placeholder --]\n[[/div]]\n[[/div]]\n\n[[div class="international-block content-panel"]]\n[[div class="panel-heading"]]\nSCP International\n[[/div]]\n[[div class="panel-body"]]\n\n[[div class="international-entry"]]\n[[div class="international-title"]]\n[[image int.png]] [http://scp-int.wikidot.com/ SCP-INT] -- International Translation Archive\n[[/div]]\n[[div class="international-description"]]\nA gathering place for the International Community of the SCP Foundation\n[[/div]]\n[[div class="international-footer"]]\nAdmins: [[user dr-grym]] -- [[user dr-grom]]\n[[/div]]\n[[/div]]\n[[/div]]\n[[div class="panel-footer"]]\n  [!-- Placeholder --]\n[[/div]]\n[[/div]]\n\n[[div class="international-block content-panel"]]\n[[div class="panel-heading"]]\nInternational Sites\n[[/div]]\n[[div class="panel-body"]]\n\n[[div class="international-entry"]]\n[[div class="international-title"]]\n[[image ru.png]] [http://scp-ru.wikidot.com/ SCP-RU] -- SCP Foundation (Russian Branch)\n[[/div]]\n[[div class="international-description"]]\nRussian Counterpart! Translation available **[http://translate.google.ca/translate?hl=en&sl=ru&tl=en&u=http%3A%2F%2Fscp-ru.wikidot.com%2F here]**.\nThe Russian site is the oldest of the translation sites active on the web.\n[[/div]]\n[[div class="international-footer"]]\nAdmins: [[user Gene R]] -- [[user Osobist]] -- [[user Resure]]\n[[/div]]\n[[/div]]\n\n[[div class="international-entry"]]\n[[div class="international-title"]]\n[[image kr.png]] [http://ko.scp-wiki.net/ SCP-KO] -- SCP 재단\n[[/div]]\n[[div class="international-description"]]\nOur Korean Colleagues!  Translation available **[http://translate.google.com/translate?sl=ko&tl=en&js=n&prev=_t&hl=en&ie=UTF-8&layout=2&eotf=1&u=http%3A%2F%2Fscpkoreahq.wikidot.com%2F here]**.\n[[/div]]\n[[div class="international-footer"]]\nAdmins: [[user Salamander724]] -- [[user QAZ135]] -- [[user shfoakdls]]\n[[/div]]\n[[/div]]\n\n[[div class="international-entry"]]\n[[div class="international-title"]]\n[[image cn.png]] [http://scp-wiki-cn.wikidot.com/ SCP-CN] -- SCP基金会\n[[/div]]\n[[div class="international-description"]]\nOur Chinese Branch! Translation available **[http://translate.google.com/translate?hl=en&sl=zh-CN&tl=en&prev=_dd&u=http%3A%2F%2Fscp-wiki-cn.wikidot.com%2F here]**.\n[[/div]]\n[[div class="international-footer"]]\nAdmins: [[user areyoucrazytom]] -- [[user SunnyClockwork]]\n[[/div]]\n[[/div]]\n\n[[div class="international-entry"]]\n[[div class="international-title"]]\n[[image fr.png]] [http://fondationscp.wikidot.com/ SCP-FR] -- Fondation SCP\n[[/div]]\n[[div class="international-description"]]\nThe French have joined! Translation available  **[http://translate.google.com/translate?sl=auto&tl=en&js=n&prev=_t&hl=en&ie=UTF-8&eotf=1&u=http%3A%2F%2Ffondationscp.wikidot.com%2F&act=url here]**.\n[[/div]]\n[[div class="international-footer"]]\nAdmins: [[user Mafiew]] -- [[user DrJohannes]] -- [[user Neremsa]]\n[[/div]]\n[[/div]]\n\n[[div class="international-entry"]]\n[[div class="international-title"]]\n[[image pl.png]] [http://scp-wiki.net.pl SCP-PL] -- SCP Polska Filia\n[[/div]]\n[[div class="international-description"]]\nThe Polish Branch of the site! Translation available **[http://translate.google.com/translate?hl=en&sl=pl&tl=en&u=http://scp-pl.wikidot.com/ here]**!\n[[/div]]\n[[div class="international-footer"]]\nAdmins: [[user Mefioo9]] -- [[user Wanna-amigo]] -- [[user Dr_Blackpeace]]\n[[/div]]\n[[/div]]\n\n[[div class="international-entry"]]\n[[div class="international-title"]]\n[[image es.png]] [http://lafundacionscp.wikidot.com/ SCP-ES] -- La Fundación SCP\n[[/div]]\n[[div class="international-description"]]\nThe Spanish Branch! Translation available **[http://translate.google.com/translate?hl=en&sl=es&tl=en&u=http%3A%2F%2Flafundacionscp.wikidot.com%2F&sandbox=1 here]**!\n[[/div]]\n[[div class="international-footer"]]\nAdmins: [[user Dr Merlin -VI]] -- [[user Dc_Yerko]]\n[[/div]]\n[[/div]]\n\n[[div class="international-entry"]]\n[[div class="international-title"]]\n[[image th.png]] [http://scp-th.wikidot.com/ SCP-TH] -- สถาบัน SCP\n[[/div]]\n[[div class="international-description"]]\nThe Thai Translators! Translation available **[http://translate.google.com/translate?hl=en&sl=th&tl=en&u=http%3A%2F%2Fscp-th.wikidot.com%2F&sandbox=1 here]**!\n[[/div]]\n[[div class="international-footer"]]\nAdmins: [[user DrSSS]] -- [[user Slang]] -- [[user Kuruni]]\n[[/div]]\n[[/div]]\n\n[[div class="international-entry"]]\n[[div class="international-title"]]\n[[image jp.png]] [http://scp-jp.wikidot.com/ SCP-JP] -- SCP財団\n[[/div]]\n[[div class="international-description"]]\nThe Japanese branch! Translation available **[http://translate.google.ca/translate?hl=en&sl=ja&tl=en&u=http%3A%2F%2Fscp-jp.wikidot.com%2F&sandbox=1 here]**!\n[[/div]]\n[[div class="international-footer"]]\nAdmins: [[user Ikr - 4185]] — [[user Nanimono Demonai]]\n[[/div]]\n[[/div]]\n\n[[div class="international-entry"]]\n[[div class="international-title"]]\n[[image de.png]] [http://scp-wiki-de.wikidot.com/ SCP-DE] -- SCP Deutschland\n[[/div]]\n[[div class="international-description"]]\nOur German branch! Translation available **[https://translate.google.com/translate?sl=auto&tl=en&js=y&prev=_t&hl=en&ie=UTF-8&u=http%3A%2F%2Fscp-wiki-de.wikidot.com%2F&edit-text=&act=url here]**!\n[[/div]]\n[[div class="international-footer"]]\nAdmins: [[user Dr_Grom]] -- [[user ThePencilWriter]]\n[[/div]]\n[[/div]]\n\n[[div class="international-entry"]]\n[[div class="international-title"]]\n[[image it.png]] [http://fondazionescp.wikidot.com/ SCP-IT] -- Fondazione SCP\n[[/div]]\n[[div class="international-description"]]\nThe Italian Stallions! Translation available **[https://translate.google.com/translate?sl=auto&tl=en&js=y&prev=_t&hl=en&ie=UTF-8&u=http%3A%2F%2Ffondazionescp.wikidot.com%2F&edit-text=&act=url here]**!\n[[/div]]\n[[div class="international-footer"]]\nAdmins: [[user Dr Voker]] — [[user Dr Pisy]] — [[user Afro Gufo]]\n[[/div]]\n[[/div]]\n\n[[div class="international-entry"]]\n[[div class="international-title"]]\n[[image ua.png]] [http://scp-ukrainian.wikidot.com/ SCP-UA] -- SCP Foundation(Фонд SCP)\n[[/div]]\n[[div class="international-description"]]\nOur Ukrainian Associates! Translation available **[https://translate.google.com/translate?sl=auto&tl=en&js=y&prev=_t&hl=en&ie=UTF-8&u=http%3A%2F%2Fscp-ukrainian.wikidot.com%2F&edit-text=&act=url here]**!\n[[/div]]\n[[div class="international-footer"]]\nAdmins: [[user murzei_chaos]] -- [[user FishStealer]] -- [[user Dr Bushroot]]\n[[/div]]\n[[/div]]\n\n[[div class="international-entry"]]\n[[div class="international-title"]]\n[[image pt.png]] [http://scp-pt-br.wikidot.com/ SCP-PT/BR] -- Lusófona Branch\n[[/div]]\n[[div class="international-description"]]\nOur Portuguese Pals! Translation available **[https://translate.google.com/translate?sl=auto&tl=en&js=y&prev=_t&hl=en&ie=UTF-8&u=http%3A%2F%2Fscp-pt-br.wikidot.com%2F&edit-text=&act=url here]**!\n[[/div]]\n[[div class="international-footer"]]\nAdmins: [[user Slashannemoo]]\n[[/div]]\n[[/div]]\n\n[[div class="international-entry"]]\n[[div class="international-title"]]\n[[image cz.png]] [http://scp-cs.wikidot.com/ SCP-CZ] -- SCP Nadace\n[[/div]]\n[[div class="international-description"]]\nOur Czech Companions! Translation available **[http://translate.google.com/translate?hl=en&sl=cs&tl=en&u=http%3A%2F%2Fscp-cs.wikidot.com%2F&sandbox=1 here]**!\n[[/div]]\n[[div class="international-footer"]]\nAdmins: [[user HawkeyeVole]]\n[[/div]]\n[[/div]]\n\n[[/div]]\n[[div class="panel-footer"]]\n  [!-- Placeholder --]\n[[/div]]\n[[/div]]\n-----\n[[size 90%]]The SCP Foundation Wiki **Sigma-9** theme and style was designed by [[user Aelanna]],\nand used under the Creative Commons Attribution-ShareAlike 3.0 license ([[[https://creativecommons.org/licenses/by-sa/3.0/ |CC-BY-SA]]]).\n[[/size]]\n\n[[html]]\n<a rel="license" href="http://creativecommons.org/licenses/by-sa/3.0/"><img alt="Creative Commons License" style="border-width:0" src="https://i.creativecommons.org/l/by-sa/3.0/88x31.png" /></a><br />This work is licensed under a <a rel="license" href="http://creativecommons.org/licenses/by-sa/3.0/">Creative Commons Attribution-ShareAlike 3.0 Unported License</a>.\n[[/html]]
178	A profile has not been set up for this user.
179	A profile has not been set up for this user.
180	[!--\nWelcome to your MyWikidot Custom Installation!\n--]\nCongratulations, you have successfully configured and launched your Wikidot custom installation!\n+ What to do next\n++ Experienced Wikidot users should [[[mywikidot-info|start here]]].\n++ Customize this wiki\nWikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n++ Customize the default templates\nThere are 2 initial default templates for other wikis. One is located at [[[template-en::start|template-en]]] and the other at [[[template-blog::start|template-blog]]]. If someone creates a new wiki, these are presented as choices and the selected template is cloned to the new wiki's address. You should customize [[[template-en::start|template-en]]] and [[[template-blog::start|template-blog]]] to suit your needs.\n++ Create more templates\nSimply create new wikis with **web site names** starting with "template-" (e.g. "template-pl", "template-recipes") and your users will be have even more choices for the basic wiki layout they want to start with. \n++ Visit Wikidot.org\nGo to **[*http://www.wikidot.org www.wikidot.org]** -- home of the Wikidot open source software -- for extra documentation, howtos, tips and support.\n++ Visit the Wikidot Community Site\nGo to **[*http://community.wikidot.com community.wikidot.com]** -- for even more tips, tricks and help from a very active community of Wikidot users.\n++ Visit the MyWikidot.local Project Site\nGo to **[*http://my-wd-local.wikidot.com/ my-wd-local.wikidot.com]** -- for tips, discussions and how-to articles.\n---------------\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org] and the developers discussion at [*http://groups.google.com/group/wikidot Wikidot dev-list].\n+ Search all wikis\n[[module SearchAll]]\n+ Search users\n[[module SearchUsers]]
181	* [# Sample Menu]\n * [[[mywikidot-info|Experienced users]]]\n * [[[mywikidot-blank|Link to a non-existing page]]]\n* [# Edit/Print]\n * [[button edit text="Edit This Page"]]\n * [[button print text="Print This Page"]]\n* [# Admin]\n * [[[nav:top|Edit Top Navigation]]]\n * [[[nav:side|Edit Side Navigation]]]\n * [[[admin:manage|Site Manager]]]\n\n[!-- top nav menu, use only one bulleted list above --]
182	* [[[start | Welcome page]]]\n\n* [[[What is a Wiki?]]]\n* [[[How to edit pages?]]]\n* [[[new-site | Get a new wiki!]]]\n\n+ All wikis\n\n* [[[system-all:activity | Recent activity]]]\n* [[[system-all:all-sites | All wikis]]]\n* [[[system-all:sites-by-tags | Wikis by tags]]]\n* [[[system-all:search | Search]]]\n\n+ This wiki\n\n* [[[system: join | How to join this site?]]]\n* [[[system:members | Site members]]]\n\n* [[[system: Recent changes]]]\n* [[[system: List all pages]]]\n* [[[system:page-tags-list|Page Tags]]]\n\n* [[[admin:manage|Site Manager]]]\n\n++ Page tags\n[[module TagCloud minFontSize="80%" maxFontSize="200%"  maxColor="8,8,64" minColor="100,100,128" target="system:page-tags" limit="30"]]\n\n++ Add a new page\n[[module NewPage size="15" button="new page"]]\n\n= [[size 80%]][[[nav:side | edit this panel]]][[/size]]
183	* [[[start | Welcome page]]]\n\n* [[[What is a Wiki Site?]]]\n* [[[How to edit pages?]]]\n\n* [[[system: join | How to join this site?]]]\n* [[[system:members | Site members]]] \n\n* [[[system: Recent changes]]]\n* [[[system: List all pages]]]\n* [[[system:page-tags-list|Page Tags]]]\n\n* [[[admin:manage|Site Manager]]]\n\n++ Page tags\n[[module TagCloud minFontSize="80%" maxFontSize="200%"  maxColor="8,8,64" minColor="100,100,128" target="system:page-tags" limit="30"]]\n\n++ Add a new page\n[[module NewPage size="15" button="new page"]]\n\n= [[size 80%]][[[nav:side | edit this panel]]][[/size]]
184	* [# Sample Menu]\n * [[[www::start|MyWikidot Home]]]\n * [[[www::mywikidot-info|Experienced users]]]\n* [# Edit/Print]\n * [[button edit text="Edit This Page"]]\n * [[button print text="Print This Page"]]\n* [# Admin]\n * [[[nav:top|Edit Top Navigation]]]\n * [[[nav:side|Edit Side Navigation]]]\n * [[[admin:manage|Site Manager]]]\n\n[!-- top nav menu, use only one bulleted list above --]
185	[[module Search]]\n\n[!-- please do not remove or change this page if you want to keep the search function working --]
187	++ If this is your first site\n\nThen there are some things you need to know:\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org].
188	[!--\nHow To Edit Pages - Quickstart\n--]\nIf you are allowed to edit pages in this Site, simply click on //edit// button at the bottom of the page. This will open an editor with a toolbar pallette with options.\n\nTo create a link to a new page, use syntax: {{``[[[new page name]]]``}} or {{``[[[new page name | text to display]]]``}}. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!\n\nAlthough creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit [*http://www.wikidot.org/doc Documentation pages] (at wikidot.org) to learn more.
189	[[module ManageSite]]
190	[[note]]\nPlease change this page according to your policy (configure first using [[[admin:manage|Site Manager]]]) and remove this note.\n[[/note]]\n\n+ Who can join?\n\nYou can write here who can become a member of this site.\n\n+ Join!\n\nSo you want to become a member of this site? Tell us why and apply now!\n\n[[module MembershipApply]] \n\nOr, if you already know a "secret password", go for it!\n\n[[module MembershipByPassword]]
191	[[module TagCloud limit="200" target="system:page-tags"]]\n\n[!--\n\nYou can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module \nBut if you want to keep the tag functionality working - do not remove these modules.\n\n--]
192	[[module SiteChanges]]
193	+ Members:\n\n[[module Members]]\n\n+ Moderators\n\n[[module Members group="moderators"]]\n\n+ Admins\n\n[[module Members group="admins"]]
194	[[div style="float:right; width: 50%;"]]\n[[module TagCloud limit="200" target="system:page-tags"]]\n[[/div]]\n[[module PagesByTag]]\n\n[!--\n\nYou can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module \nBut if you want to keep the tag functionality working - do not remove these modules.\n\n--]
195	[[module Pages preview="true"]]
196	No profile has been set up yet for this user.
197	[[module CSS]]\n#page-title { color:green; }\n[[/module]]\n\n[!--\nWelcome to your MyWikidot Custom Installation!\n--]\nCongratulations, you have successfully configured and launched your Wikidot custom installation!\n+ What to do next\n++ Experienced Wikidot users should [[[mywikidot-info|start here]]].\n++ Customize this wiki\nWikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n++ Customize the default templates\nThere are 2 initial default templates for other wikis. One is located at [[[template-en::start|template-en]]] and the other at [[[template-blog::start|template-blog]]]. If someone creates a new wiki, these are presented as choices and the selected template is cloned to the new wiki's address. You should customize [[[template-en::start|template-en]]] and [[[template-blog::start|template-blog]]] to suit your needs.\n++ Create more templates\nSimply create new wikis with **web site names** starting with "template-" (e.g. "template-pl", "template-recipes") and your users will be have even more choices for the basic wiki layout they want to start with. \n++ Visit Wikidot.org\nGo to **[*http://www.wikidot.org www.wikidot.org]** -- home of the Wikidot open source software -- for extra documentation, howtos, tips and support.\n++ Visit the Wikidot Community Site\nGo to **[*http://community.wikidot.com community.wikidot.com]** -- for even more tips, tricks and help from a very active community of Wikidot users.\n++ Visit the MyWikidot.local Project Site\nGo to **[*http://my-wd-local.wikidot.com/ my-wd-local.wikidot.com]** -- for tips, discussions and how-to articles.\n---------------\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org] and the developers discussion at [*http://groups.google.com/group/wikidot Wikidot dev-list].\n+ Search all wikis\n[[module SearchAll]]\n+ Search users\n[[module SearchUsers]]
198	[[module CSS]]\n.wiki-content-table td:nth-last-child(1){\n    width: 3ch;\n}\n.wiki-content-table td:nth-last-child(2){\n    width: 12ch;\n}\n.content-panel .content-type-title > p {\n    margin: 0;\n}\n.content-panel .content-type-title {\n    /*font-size: 120%;*/\n    font-weight: bold;\n    padding: 2px 10px;\n    background-color: #666;\n    color: #fff;\n    border-radius: 9px 9px 0 0;\n    box-shadow: inset 0 1px rgba(255,255,255,0.3),\n                inset 0 10px rgba(255,255,255,0.2),\n                inset 0 10px 20px rgba(255,255,255,0.25);\n}\n.content-panel .content-type-description {\n    padding: 0;\n    text-align: justify;\n}\n\n.content-panel .wiki-content-table {\n    margin: 2px 0 0 0;\n    width: 100%;\n    word-break: break-word;\n}\n.content-panel .wiki-content-table th:nth-of-type(2),\n.content-panel .wiki-content-table th:nth-of-type(3),\n.content-panel .wiki-content-table td:nth-of-type(2),\n.content-panel .wiki-content-table td:nth-of-type(3) {\n    text-align: center;\n}\n.content-panel .wiki-content-table th,\n.content-panel .wiki-content-table td {\n    border: none;\n}\n.content-panel .wiki-content-table tr {\n    border-bottom: solid 1px rgba(0,0,0,0.1);\n}\n.content-panel .wiki-content-table th {\n    border: 2px solid white;\n    border-width: 0 2px;\n    background-color: #666;\n    color: white;\n}\n.desktop-only { display: block; }\n.mobile-only { display: none; }\n@media (max-width: 979px) {\n    .desktop-only { display: none; }\n    .mobile-only { display: block; }\n}\n[[/module]]\n\n[!-- SCP Block --]\n\n[[div class="feature-block desktop-only"]]\n\nA list of all newly created pages in one list can be found [[[most-recently-created| here]]].\n\n[[div class="content-panel standalone left-column"]]\n[[div class="content-type-title"]]\nNewly Created SCPs\n[[/div]]\n[[div class="content-type-description"]]\n\n[[module ListPages rating=">=-10" name="scp-*" tags="-001-proposal -admin -artwork -author -contest -creepypasta -essay -former-author -goi-format -guide -hub -in-deletion -in-rewrite -news -orphaned -project -sandbox -site -splash -supplement -tale -workbench" order="created_at desc" separate="false"  perPage="51" limit="255" urlAttrPrefix="scps" prependLine="||~ Page ||~ [[size 0%]]Date Created[[/size]]📆︎ ||~ [[size 0%]]Comments[[/size]]💬︎ ||"]]\n|| %%title_linked%% || %%created_at|%e%%@<&nbsp;>@%%created_at|%b %R%% || %%comments%% ||\n[[/module]]\n[[/div]]\n[[div class="panel-footer"]]\n  [!-- Placeholder --]\n[[/div]]\n[[/div]]\n\n[!-- Tale/GoI format/Art Block --]\n\n[[div class="content-panel standalone right-column"]]\n[[div class="content-type-title"]]\nTales, GoI Formats, and Artwork Pages\n[[/div]]\n[[div class="content-type-description"]]\n\n[[module ListPages rating=">=-10" tags="tale goi-format artwork 001-proposal" order="created_at desc" separate="false" perPage="35" limit="150" urlAttrPrefix="tale_goi_art" prependLine="||~ Page ||~ [[size 0%]]Date Created[[/size]]📆︎ ||~ [[size 0%]]Comments[[/size]]💬︎ ||"]]\n|| %%title_linked%% || %%created_at|%e%%@<&nbsp;>@%%created_at|%b %R%% || %%comments%% ||\n[[/module]]\n[[/div]]\n[[div class="panel-footer"]]\n  [!-- Placeholder --]\n[[/div]]\n[[/div]]\n\n~~~~\n\n[!-- Other Block --]\n\n\n[[div class="content-panel standalone left-column"]]\n[[div class="content-type-title"]]\nOther Types of Pages\n[[/div]]\n[[div class="content-type-description"]]\n\n[[module ListPages rating=">=-10" category="*" tags="-scp -tale -goi-format -in-deletion -in-rewrite -artwork admin archived author contest creepypasta essay guide hub news orphaned project sandbox site splash supplement template theme workbench " order="created_at desc" separate="false" perPage="35" limit="150" urlAttrPrefix="other" prependLine="||~ Page ||~ [[size 0%]]Date Created[[/size]]📆︎ ||~ [[size 0%]]Comments[[/size]]💬︎ ||"]]\n|| %%title_linked%% || %%created_at|%e%%@<&nbsp;>@%%created_at|%b %R%% || %%comments%% ||\n[[/module]]\n[[/div]]\n[[div class="panel-footer"]]\n  [!-- Placeholder --]\n[[/div]]\n[[/div]]\n\n[!-- Untagged Block --]\n\n[[div class="content-panel standalone right-column"]]\n[[div class="content-type-title"]]\nUntagged Pages\n[[/div]]\n[[div class="content-type-description"]]\n\n[[module ListPages rating=">=-10" tags="-admin -archived -artwork -author -contest -creepypasta -essay -fragment -goi-format -guide -hub -in-deletion -in-rewrite -news -orphaned -project -sandbox -scp -site -splash -supplement -tale -workbench" order="created_at desc" separate="false" perPage="50" limit="150" urlAttrPrefix="untagged" prependLine="||~ Page ||~ [[size 0%]]Date Created[[/size]]📆︎ ||~ [[size 0%]]Comments[[/size]]💬︎ ||"]]\n|| %%title_linked%% || %%created_at|%e%%@<&nbsp;>@%%created_at|%b %R%% || %%comments%% ||\n[[/module]]\n[[/div]]\n[[div class="panel-footer"]]\n  [!-- Placeholder --]\n[[/div]]\n[[/div]]\n\n~~~~\n\n[[/div]]\n\n[!-- All pages, mobile only --]\n\n[[div class="feature-block mobile-only"]]\n\n[[div class="content-panel standalone"]]\n[[div class="content-type-title"]]\nAll Newly Created Pages\n[[/div]]\n[[div class="content-type-description"]]\n\n[[module ListPages rating=">=-10" order="created_at desc" separate="false"  perPage="30" prependLine="||~ Page ||~ [[size 0%]]Date Created[[/size]]📆︎ ||~ [[size 0%]]Comments[[/size]]💬︎ ||"]]\n|| %%title_linked%% || %%created_at|%e%%@<&nbsp;>@%%created_at|%b %R%% || %%comments%% ||\n[[/module]]\n[[/div]]\n[[div class="panel-footer"]]\n  [!-- Placeholder --]\n[[/div]]\n[[/div]]\n\n~~~~\n\n\n[[/div]]
199	[[module CSS]]\n.content-panel .content-type-title > p {\n  margin: 0;\n}\n.content-panel .content-type-title {\n  font-size: 120%;\n  font-weight: bold;\n  padding: 5px 20px;\n  background-color: #666;\n  color: #fff;\n  border-radius: 8px 8px 0 0;\n  box-shadow: inset 0 1px 1px rgba(255,255,255,.8),\n    inset 0 15px 1px rgba(255,255,255,.2),\n    inset 0 15px 10px rgba(255,255,255,.2);\n}\n.content-panel .content-type-description {\npadding: 0 20px;\ntext-align: justify;\n}\n.content-panel.content-row .content-type-description {\npadding: 5px 20px;\n}\n.content-panel.content-row .content-type-description-2 {\npadding: 0 20px;\ntext-align: justify;\n}\n[[/module]]\n\nListed here are the lowest rated active pages on the entire site. Please read them and leave ideas or thoughts on how to improve them in the discussion threads.\n\nFor reference on SCPs in the single digits, refer to [[[Lowest Rated SCPs]]]. For references on other articles in the single digits, refer to [[[Lowest Rated Articles]]].\n\n[[div class="content-panel standalone content-row"]]\n[[div class="content-type-title"]]\nLowest Rated Pages\n[[/div]]\n\n[[div class="content-type-description-2"]]\n\n[[table style="width: 100%;"]]\n[[row style="font-weight: bold; color: #fff; background-color: #666;"]]\n[[cell style="padding: 0 2px; width: 40%;"]]\nPage\n[[/cell]]\n[[cell style="padding: 0 2px; width: 15%; text-align: center;"]]\nRating\n[[/cell]]\n[[cell style="padding: 0 2px; width: 15%; text-align: center;"]]\nComments\n[[/cell]]\n[[cell style="padding: 0 2px; width: 30%; text-align: center;"]]\nDate Created\n[[/cell]]\n[[/row]]\n[[/table]]\n\n[[/div]]\n\n[[div class="content-type-description"]]\n\n[[module ListPages order="ratingAsc" limit="100" tags="-archived, -admin, -author, -sandbox, -in-deletion, -in-rewrite" rating="<-0" perPage="100" prependLine="[[include component:listpages-table-alt]]" appendLine="[[/table]]" separate="false"]]\n[[row]]\n[[cell style="vertical-align: top;"]]\n%%title_linked%%\n[[/cell]]\n[[cell style="vertical-align: top; text-align: center;"]]\n%%rating%%\n[[/cell]]\n[[cell style="vertical-align: top; text-align: center;"]]\n%%comments%%\n[[/cell]]\n[[cell style="vertical-align: top; text-align: center;"]]\n%%created_at%%\n[[/cell]]\n[[/row]]\n[[/module]]\n[[/div]]\n[[/div]]\n~~~~~~~~\n----\n [[div class="content-panel standalone content-row"]]\n[[div class="content-type-title"]]\nPages In Deletion\n[[/div]]\n\n[[div class="content-type-description-2"]]\n\n[[table style="width: 100%;"]]\n[[row style="font-weight: bold; color: #fff; background-color: #666;"]]\n[[cell style="padding: 0 2px; width: 30%;"]]\nPage\n[[/cell]]\n[[cell style="padding: 0 2px; width: 15%; text-align: center;"]]\nRating\n[[/cell]]\n[[cell style="padding: 0 2px; width: 15%; text-align: center;"]]\nComments\n[[/cell]]\n[[cell style="padding: 0 2px; width: 20%; text-align: center;"]]\nDate Created\n[[/cell]]\n[[cell style="padding: 0 2px; width: 20%; text-align: center;"]]\nLast Edit\n[[/cell]]\n[[/row]]\n[[/table]]\n\n[[/div]]\n\n[[div class="content-type-description"]]\n\n[[module ListPages order="ratingAsc" limit="100" tags="in-deletion" perPage="100" prependLine="[[include component:listpages-table-alt-2]]" appendLine="[[/table]]" separate="false"]]\n[[row]]\n[[cell style="vertical-align: top;"]]\n%%title_linked%%\n[[/cell]]\n[[cell style="vertical-align: top; text-align: center;"]]\n%%rating%%\n[[/cell]]\n[[cell style="vertical-align: top; text-align: center;"]]\n%%comments%%\n[[/cell]]\n[[cell style="vertical-align: top; text-align: center;"]]\n%%created_at%%\n[[/cell]]\n[[cell style="vertical-align: top; text-align: center;"]]\n%%updated_at%%\n[[/cell]]\n[[/row]]\n[[/module]]\n\n[[/div]]\n[[/div]]\n~~~~~~~~\n----\n[[div class="content-panel standalone content-row"]]\n[[div class="content-type-title"]]\nPages In Rewrite\n[[/div]]\n\n[[div class="content-type-description-2"]]\n\n[[table style="width: 100%;"]]\n[[row style="font-weight: bold; color: #fff; background-color: #666;"]]\n[[cell style="padding: 0 2px; width: 40%;"]]\nPage\n[[/cell]]\n[[cell style="padding: 0 2px; width: 15%; text-align: center;"]]\nRating\n[[/cell]]\n[[cell style="padding: 0 2px; width: 15%; text-align: center;"]]\nComments\n[[/cell]]\n[[cell style="padding: 0 2px; width: 30%; text-align: center;"]]\nDate Created\n[[/cell]]\n[[/row]]\n[[/table]]\n\n[[/div]]\n\n[[div class="content-type-description"]]\n\n[[module ListPages order="created_atDsc" limit="100" tags="in-rewrite" perPage="100" prependLine="[[include component:listpages-table-alt]]" appendLine="[[/table]]" separate="false"]]\n[[row]]\n[[cell style="vertical-align: top;"]]\n%%title_linked%%\n[[/cell]]\n[[cell style="vertical-align: top; text-align: center;"]]\n%%rating%%\n[[/cell]]\n[[cell style="vertical-align: top; text-align: center;"]]\n%%comments%%\n[[/cell]]\n[[cell style="vertical-align: top; text-align: center;"]]\n%%created_at%%\n[[/cell]]\n[[/row]]\n[[/module]]\n\n[[/div]]\n[[/div]]\n\n~~~~~~~~\n----\n[[div class="content-panel standalone content-row"]]\n[[div class="content-type-title"]]\nUntagged Pages\n[[/div]]\n\n[[div class="content-type-description-2"]]\n\n[[table style="width: 100%;"]]\n[[row style="font-weight: bold; color: #fff; background-color: #666;"]]\n[[cell style="padding: 0 2px; width: 40%;"]]\nPage\n[[/cell]]\n[[cell style="padding: 0 2px; width: 15%; text-align: center;"]]\nRating\n[[/cell]]\n[[cell style="padding: 0 2px; width: 15%; text-align: center;"]]\nComments\n[[/cell]]\n[[cell style="padding: 0 2px; width: 30%; text-align: center;"]]\nDate Created\n[[/cell]]\n[[/row]]\n[[/table]]\n\n[[/div]]\n\n[[div class="content-type-description"]]\n\n[[module ListPages rating=">=-10" order="created_atDsc" limit="100"  tags="-admin -archived -artwork -author -contest -creepypasta -essay -fragment -goi-format -guide -hub -in-deletion -in-rewrite -news -orphaned -project -sandbox -scp -site -splash -supplement -tale -workbench" perPage="100" prependLine="[[include component:listpages-table-alt]]" appendLine="[[/table]]" separate="false"]]\n[[row]]\n[[cell style="vertical-align: top;"]]\n%%title_linked%%\n[[/cell]]\n[[cell style="vertical-align: top; text-align: center;"]]\n%%rating%%\n[[/cell]]\n[[cell style="vertical-align: top; text-align: center;"]]\n%%comments%%\n[[/cell]]\n[[cell style="vertical-align: top; text-align: center;"]]\n%%created_at%%\n[[/cell]]\n[[/row]]\n[[/module]]\n\n[[/div]]\n\n[[/div]]\n~~~~~~~~\n----\n[[div class="content-panel standalone content-row"]]\n[[div class="content-type-title"]]\nImproper Deletions\n[[/div]]\n\n[[div class="content-type-description-2"]]\n\n[[table style="width: 100%;"]]\n[[row style="font-weight: bold; color: #fff; background-color: #666;"]]\n[[cell style="padding: 0 2px; width: 40%;"]]\nPage\n[[/cell]]\n[[cell style="padding: 0 2px; width: 15%; text-align: center;"]]\nRating\n[[/cell]]\n[[cell style="padding: 0 2px; width: 15%; text-align: center;"]]\nComments\n[[/cell]]\n[[cell style="padding: 0 2px; width: 30%; text-align: center;"]]\nDate Created\n[[/cell]]\n[[/row]]\n[[/table]]\n\n[[/div]]\n\n[[div class="content-type-description"]]\n\n[[module ListPages category="deleted" limit="100" perPage="100" prependLine="[[include component:listpages-table-alt]]" appendLine="[[/table]]" separate="false"]]\n[[row]]\n[[cell style="vertical-align: top;"]]\n%%title_linked%%\n[[/cell]]\n[[cell style="vertical-align: top; text-align: center;"]]\n%%rating%%\n[[/cell]]\n[[cell style="vertical-align: top; text-align: center;"]]\n%%comments%%\n[[/cell]]\n[[cell style="vertical-align: top; text-align: center;"]]\n%%created_at%%\n[[/cell]]\n[[/row]]\n[[/module]]\n[[/div]]\n[[/div]]
200	%%content%%
201	[[=]]\n+ This page doesn't exist yet!\n[[/=]]\n\n----\n\n[[div style="background-color: #600; border: solid 1px #600; border-radius: 20px; color: #fff; width: 450px; margin: 0 auto; font-size: 150%; text-align: center; box-shadow: 0 2px 6px rgba(0,0,0,.5), inset 0 1px rgba(255,255,255,.5), inset 0 10px rgba(255,204,204,.5), inset 0 10px 20px rgba(255,204,204,.3), inset 0 -15px 30px rgba(48,0,0,.5); line-height: 100%; padding: 0 10px;"]]\n**Did you get a code review first?**\n[[/div]]\n\n[[div style="background-color: #fff0f0; border: solid 1px #600; border-radius: 20px; color: #300; width: 450px; margin: 20px auto 0; text-align: center; box-shadow: 0 2px 6px rgba(0,0,0,.5); padding: 0 10px;"]]\nWikijump has many resources to help you learn about the platform, and pull reviews that have been critiqued are far more likely to be successful.\n----\n**[https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/creating-a-pull-request How to Create a Pull Request]]] -- [https://github.com/scpwiki/wikidot GitHub] -- [https://scuttle.atlassian.net JIRA]**\n\n**[[[Image Use Policy]]]**\n\n**[/forum/c-89000 Help Forum: Ideas Critique]**\n**[/forum/c-50864 Help Forum: Drafts Critique]**\n**[[[chat-guide|Chat Guide]]]**\n[[/div]]\n\n[[div style="background-color: #fffff0; border: solid 1px #660; border-radius: 20px; color: #330; width: 450px; margin: 20px auto 0; text-align: center; box-shadow: 0 2px 6px rgba(0,0,0,.5); padding: 0 10px;"]]\n[[size 120%]]Remember: **The staging site is for testing and development, not feedback, critique, or posting finished work.**[[/size]]\n----\nIt is your responsibility to help contribute to Wikijump. Wikidot servers are not required to justify their crashes or errors.\n\nIf you understand all of the above and still wish to create this page, **[[button edit text="click here"]]** to do so.\n[[/div]]
202	[[=]]\n+ This page doesn't exist yet!\n[[/=]]\n\n----\n\n[[div style="background-color: #600; border: solid 1px #600; border-radius: 20px; color: #fff; width: 450px; margin: 0 auto; font-size: 150%; text-align: center; box-shadow: 0 2px 6px rgba(0,0,0,.5), inset 0 1px rgba(255,255,255,.5), inset 0 10px rgba(255,204,204,.5), inset 0 10px 20px rgba(255,204,204,.3), inset 0 -15px 30px rgba(48,0,0,.5); line-height: 100%; padding: 0 10px;"]]\n**Did you get a code review first?**\n[[/div]]\n\n[[div style="background-color: #fff0f0; border: solid 1px #600; border-radius: 20px; color: #300; width: 450px; margin: 20px auto 0; text-align: center; box-shadow: 0 2px 6px rgba(0,0,0,.5); padding: 0 10px;"]]\nWikijump has many resources to help you learn about the platform, and pull reviews that have been critiqued are far more likely to be successful.\n----\n**[https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/creating-a-pull-request How to Create a Pull Request] -- [https://github.com/scpwiki/wikidot GitHub] -- [https://scuttle.atlassian.net JIRA]**\n\n**[[[Image Use Policy]]]**\n\n**[/forum/c-89000 Help Forum: Ideas Critique]**\n**[/forum/c-50864 Help Forum: Drafts Critique]**\n**[[[chat-guide|Chat Guide]]]**\n[[/div]]\n\n[[div style="background-color: #fffff0; border: solid 1px #660; border-radius: 20px; color: #330; width: 450px; margin: 20px auto 0; text-align: center; box-shadow: 0 2px 6px rgba(0,0,0,.5); padding: 0 10px;"]]\n[[size 120%]]Remember: **The staging site is for testing and development, not feedback, critique, or posting finished work.**[[/size]]\n----\nIt is your responsibility to help contribute to Wikijump. Wikidot servers are not required to justify their crashes or errors.\n\nIf you understand all of the above and still wish to create this page, **[[button edit text="click here"]]** to do so.\n[[/div]]
203	[!-- copied from http://www.scp-wiki.net/theme:black-highlighter-theme --]\n\n[[iftags +theme -nobhl]]\n[[module css]]\n.scp-image-block.block-right {\n    padding: 1em !important;\n    background: var(--gradient-header);\n}\n\n.scp-image-block img {\n    padding-bottom: 1em;\n}\n\n.colors_container {\n    width: 100%;\n    display: -webkit-box;\n    display: -webkit-flex;\n    display: -moz-box;\n    display: -ms-flexbox;\n    display: flex;\n    -webkit-box-orient: vertical;\n    -webkit-box-direction: normal;\n    -webkit-flex-direction: column;\n       -moz-box-orient: vertical;\n       -moz-box-direction: normal;\n        -ms-flex-direction: column;\n            flex-direction: column;\n    margin: 0 auto;\n    font-weight: 700;\n    font-family: var(--header-font);\n}\n\n.colors_container > .colors,\n.colors_container > .subcolors {\n    width: 100%;\n    -webkit-flex-shrink: 0;\n        -ms-flex-negative: 0;\n            flex-shrink: 0;\n    display: -webkit-box;\n    display: -webkit-flex;\n    display: -moz-box;\n    display: -ms-flexbox;\n    display: flex;\n    -webkit-flex-wrap: wrap;\n        -ms-flex-wrap: wrap;\n            flex-wrap: wrap;\n}\n\n.colors > .color,\n.subcolors > .color {\n    display: -webkit-box;\n    display: -webkit-flex;\n    display: -moz-box;\n    display: -ms-flexbox;\n    display: flex;\n    -webkit-box-orient: horizontal;\n    -webkit-box-direction: normal;\n    -webkit-flex-direction: row;\n       -moz-box-orient: horizontal;\n       -moz-box-direction: normal;\n        -ms-flex-direction: row;\n            flex-direction: row;\n    -webkit-box-flex: 2;\n    -webkit-flex-grow: 2;\n       -moz-box-flex: 2;\n        -ms-flex-positive: 2;\n            flex-grow: 2;\n    -webkit-box-pack: center;\n    -webkit-justify-content: center;\n       -moz-box-pack: center;\n        -ms-flex-pack: center;\n            justify-content: center;\n    -webkit-box-align: end;\n    -webkit-align-items: flex-end;\n       -moz-box-align: end;\n        -ms-flex-align: end;\n            align-items: flex-end;\n    padding: 0.5rem;\n    margin: 0.5rem;\n}\n\n.colors > .color > .sub,\n.subcolors > .color > .sub,\n.colors > .color > .sub > .css-variable,\n.subcolors > .color > .sub > .css-variable {\n    display: -webkit-box;\n    display: -webkit-flex;\n    display: -moz-box;\n    display: -ms-flexbox;\n    display: flex;\n    -webkit-box-orient: vertical;\n    -webkit-box-direction: normal;\n    -webkit-flex-direction: column;\n       -moz-box-orient: vertical;\n       -moz-box-direction: normal;\n        -ms-flex-direction: column;\n            flex-direction: column;\n    -webkit-box-pack: center;\n    -webkit-justify-content: center;\n       -moz-box-pack: center;\n        -ms-flex-pack: center;\n            justify-content: center;\n    -webkit-box-align: center;\n    -webkit-align-items: center;\n       -moz-box-align: center;\n        -ms-flex-align: center;\n            align-items: center;\n    width: 100%;\n}\n\n.colors > .color {\n    height: 7rem;\n    -webkit-flex-basis: -webkit-calc((100%/2) - 2rem);\n        -ms-flex-preferred-size: calc((100%/2) - 2rem);\n            flex-basis: -moz-calc((100%/2) - 2rem);\n            flex-basis: calc((100%/2) - 2rem);\n}\n\n.colors > .color.one {\n    background-color: rgba(var(--gray-monochrome), 1);\n}\n\n.colors > .color.two {\n    background-color: rgba(var(--bright-accent), 1);\n}\n\n.subcolors > .color {\n    height: 4rem;\n    font-size: 75%;\n    text-align: center;\n    -webkit-flex-basis: -webkit-calc((100%/6) - 2rem);\n        -ms-flex-preferred-size: calc((100%/6) - 2rem);\n            flex-basis: -moz-calc((100%/6) - 2rem);\n            flex-basis: calc((100%/6) - 2rem);\n}\n\n.colors > .color.one,\n.colors > .color.two,\n.subcolors > .color.three,\n.subcolors > .color.four,\n.subcolors > .color.five {\n    color: rgba(var(--swatch-text-light), 1);\n}\n\n.subcolors > .color.one {\n    background-color: rgba(var(--very-light-gray-monochrome), 1);\n}\n\n.subcolors > .color.two {\n    background-color: rgba(var(--pale-gray-monochrome), 1);\n}\n\n.subcolors > .color.three {\n    background-color: rgba(var(--dark-gray-monochrome), 1);\n}\n\n.subcolors > .color.four {\n    background-color: rgba(var(--medium-accent), 1);\n}\n\n.subcolors > .color.five {\n    background-color: rgba(var(--dark-accent), 1);\n}\n\n.subcolors > .color.six {\n    background-color: rgba(var(--newpage-color), 1);\n}\n\n.color > .sub > .css-variable {\n    font-size: 75%;\n    letter-spacing: 0.1em;\n    font-family: var(--body-font);\n}\n\n.status {\n    font-family: var(--title-font);\n    font-weight: 900;\n    font-size: 200%;\n    display: flex:\n    align-content: center;\n    justify-items: center;\n    text-align: center;\n}\n\n.status span.active {\n    color: rgb(var(--white-monochrome));\n    -webkit-box-shadow:\n        inset 100vw 0 0 0 rgb(var(--rating-module-button-plus-color)),\n        0.25rem 0.25rem 0 rgb(var(--rating-module-button-plus-color)),\n        -0.25rem -0.25rem 0 rgb(var(--rating-module-button-plus-color)),\n        -0.25rem 0.25rem 0 rgb(var(--rating-module-button-plus-color)),\n        0.25rem -0.25rem 0 rgb(var(--rating-module-button-plus-color));\n        -moz-box-shadow:\n            inset 100vw 0 0 0 rgb(var(--rating-module-button-plus-color)),\n            0.25rem 0.25rem 0 rgb(var(--rating-module-button-plus-color)),\n            -0.25rem -0.25rem 0 rgb(var(--rating-module-button-plus-color)),\n            -0.25rem 0.25rem 0 rgb(var(--rating-module-button-plus-color)),\n            0.25rem -0.25rem 0 rgb(var(--rating-module-button-plus-color));\n                box-shadow:\n                    inset 100vw 0 0 0 rgb(var(--rating-module-button-plus-color)),\n                    0.25rem 0.25rem 0 rgb(var(--rating-module-button-plus-color)),\n                    -0.25rem -0.25rem 0 rgb(var(--rating-module-button-plus-color)),\n                    -0.25rem 0.25rem 0 rgb(var(--rating-module-button-plus-color)),\n                    0.25rem -0.25rem 0 rgb(var(--rating-module-button-plus-color));\n}\n\n[[/module]]\n[[>]]\n[[module Rate]]\n[[/>]]\n[[image http://scp-wiki.wdfiles.com/local--files/component%3Ablack-highlighter-theme-dev/black-highlighter-logo.svg]]\n\n[[=]]\n[[div_ class="status"]]\nThis component is currently [[span class="active"]]ACTIVE[[/span]]\n[[/div]]\n[[/=]]\n _\n[[div style="border: 1px solid #ddd; padding: 1em;"]]\n\n+ Usage\n\nOn any wiki:\n\n[[div class="code"]]\n@@[[include :scp-wiki:theme:black-highlighter-theme]]@@\n[[/div]]\n\n[[div class="blockquote"]]\n++ Optional Addons\n----\n+++ [http://www.scp-wiki.net/component:bhl-dark-sidebar Dark Sidebar]\n[[div class="code"]]\n@@[[include :scp-wiki:component:bhl-dark-sidebar]]@@\n[[/div]]\n\n+++ [http://www.scp-wiki.net/component:collapsible-sidebar Collapsible Sidebar]\n[[div class="code"]]\n@@[[include :scp-wiki:component:collapsible-sidebar]]@@\n[[/div]]\n[[/div]]\n\n[[/div]]\n\n\n\n+ What this is\n\nA component that applies the [http://scptestwiki.wikidot.com/ Black Highlighter] theme to your article.\n\nThis component will apply a stable version of the Black Highlighter theme, but it might break sometimes as it's updated.\n\n+ Reporting problems\n\nIf you've got a Github account, create an Issue [https://github.com/Nu-SCPTheme/Black-Highlighter/issues here] detailing your problem (whether it's technical, or aesthetic, or whatever).\n\nIf you don't have a Github account, or if you'd prefer to discuss your issues with someone directly, either join the {{#black-highlighter}} channel on SynIRC, or send a PM to [[*user Woedenaz]] or [[*user Croquembouche]].\n\n-----\n\n[[=]]\n+ Theme Colors\n[[/=]]\n\n[[div_ class="colors_container"]]\n[[div_ class="colors"]]\n[[div_ class="color one"]]\n[[div_ class="color sub"]]\nPayne's Grey [[span class="css-variable"]]@@gray-monochrome@@[[/span]][[span class="css-variable"]]@@(66, 66, 72)@@[[/span]]\n[[/div]]\n[[/div]]\n[[div_ class="color two"]]\n[[div_ class="color sub"]]\nRosewood [[span class="css-variable"]]@@bright-accent@@[[/span]][[span class="css-variable"]]@@(133, 0, 5)@@[[/span]]\n[[/div]]\n[[/div]]\n[[/div]]\n[[div_ class="subcolors"]]\n[[div_ class="color one"]]\n[[div_ class="color sub"]]\nAlto [[span class="css-variable"]]@@very-light-gray-monochrome@@[[/span]][[span class="css-variable"]]@@(215, 215, 215)@@[[/span]]\n[[/div]]\n[[/div]]\n[[div_ class="color two"]]\n[[div_ class="color sub"]]\nWhite Smoke [[span class="css-variable"]]@@pale-gray-monochrome@@[[/span]][[span class="css-variable"]]@@(244, 244, 244)@@[[/span]]\n[[/div]]\n[[/div]]\n[[div_ class="color three"]]\n[[div_ class="color sub"]]\nBastille [[span class="css-variable"]]@@dark-gray-monochrome@@[[/span]][[span class="css-variable"]]@@(48, 48, 52)@@[[/span]]\n[[/div]]\n[[/div]]\n[[div_ class="color four"]]\n[[div_ class="color sub"]]\nBuccaneer [[span class="css-variable"]]@@medium-accent@@[[/span]][[span class="css-variable"]]@@(100, 46, 44)@@[[/span]]\n[[/div]]\n[[/div]]\n[[div_ class="color five"]]\n[[div_ class="color sub"]]\nMaroon [[span class="css-variable"]]@@dark-accent@@[[/span]][[span class="css-variable"]]@@(100, 3, 15)@@[[/span]]\n[[/div]]\n[[/div]]\n[[div_ class="color six"]]\n[[div_ class="color sub"]]\nMango Tango [[span class="css-variable"]]@@newpage-color@@[[/span]][[span class="css-variable"]]@@(221, 102, 17)@@[[/span]]\n[[/div]]\n[[/div]]\n[[/div]]\n[[/div]]\n\n+ Examples\n\n[[include component:image-block name=https://nu-scptheme.github.io/Black-Highlighter/img/logo.svg|caption=SCP Foundation Logo|width=200px]]\n\nA horizontal rule can be created with 5 hyphens "@@-----@@" and extends across the whole page if it's not placed inside anything (eg a blockquote). The lines separating sections of this document are horizontal rules.\n\n-----\n\nTitles can be created by putting between one and six plus "+" at the start of the line\n\n[[collapsible show="+ Titles" hide="- Titles"]]\n+ First Title\n\n++ Second Title\n\n+++ Third Title\n\n++++ Fourth Title\n\n+++++ Fifth Title\n\n++++++ Sixth Title\n[[/collapsible]]\n\n@@ @@\n\n[[tabview]]\n[[tab Tabulator]]\nThis is a tab view.\n[[/tab]]\n[[tab Tabulation]]\nHey look, more text here.\n\nHow quaint.\n[[/tab]]\n[[tab Long Tab]]\nThis is a long tab. It contains a lot of text.\n\nThis is a long tab. It contains a lot of text.\n\nThis is a long tab. It contains a lot of text.\n\nThis is a long tab. It contains a lot of text.\n\nThis is a long tab. It contains a lot of text.\n\nThis is a long tab. It contains a lot of text.\n\nThis is a long tab. It contains a lot of text.\n\nThis is a long tab. It contains a lot of text.\n\nThis is a long tab. It contains a lot of text.\n\nThis is a long tab. It contains a lot of text.\n\nThis is a long tab. It contains a lot of text.\n\nThis is a long tab. It contains a lot of text.\n[[/tab]]\n[[tab This empty tab has a really long name for some odd reason. I wonder why? It is very strange.]]\n[[/tab]]\n[[tab Empty Tab]]\n[[/tab]]\n[[tab Empty Tab]]\n[[/tab]]\n[[tab Empty Tab]]\n[[/tab]]\n[[tab Empty Tab]]\n[[/tab]]\n[[tab Empty Tab]]\n[[/tab]]\n[[/tabview]]\n\n> This is a blockquote, created by putting "> " at the start of each line.\n>\n> More text\n> -----\n> That's a horizontal rule\n>\n>> Nested blockquotes\n\n||~ This is a ||~ table ||\n||You should know || how to make these ||\n||||already ||\n\n[[=]]\n[[span style="font-family:var(--body-font); font-size: calc(var(--base-font-size) * 1.25);"]]The body font is Freight Sans.[[/span]]\n[[span style="font-family:var(--header-font); font-size: calc(var(--base-font-size) * 1.25);"]]The header and title font is [https://fonts.google.com/specimen/Poppins Poppins].[[/span]]\n[[span style="font-family:var(--mono-font); font-size: calc(var(--base-font-size) * 1.25);"]]The monospace font is [https://fonts.google.com/specimen/Space+Mono Space Mono].[[/span]]\n[[/=]]\n\n-----\n[[/iftags]]\n\n[[module CSS]]\n@import url("https://nu-scptheme.github.io/Black-Highlighter/css/min/normalize.min.css");\n@import url("https://nu-scptheme.github.io/Black-Highlighter/css/min/black-highlighter.min.css");\n[[/module]]
204	[[=]]\n+ This page doesn't exist yet!\n[[/=]]\n\n----\n\n[[div style="background-color: #000; border: solid 1px #000; border-radius: 20px; color: #fff; width: 450px; margin: 0 auto; font-size: 150%; text-align: center; box-shadow: 0 2px 6px rgba(0,0,0,.5), inset 0 1px rgba(255,255,255,.5), inset 0 10px rgba(204,204,204,.5), inset 0 10px 20px rgba(204,204,204,.3), inset 0 -15px 30px rgba(0,0,0,.5); line-height: 100%; padding: 0 10px;"]]\n**Did you get approval?**\n[[/div]]\n\n[[div style="background-color: #f0f0f0; border: solid 1px #000; border-radius: 20px; width: 450px; margin: 20px auto 0; text-align: center; box-shadow: 0 2px 6px rgba(0,0,0,.5); padding: 0 10px;"]]\nYou are currently attempting to create a {{theme:}} page. The creation of theme pages is monitored by the Technical Team, as per the **[/css-policy CSS Policy]**.\n\nPlease review the CSS Policy to ensure that your theme is compliant with all portions of the CSS Policy, and get approval from an Operational or higher member of the **[http://05command.wikidot.com/technical-staff-main Technical Team]**.\n[[/div]]\n\n[[div style="background-color: #f1f1f1; border: solid 1px #000; border-radius: 20px; width: 450px; margin: 20px auto 0; text-align: center; box-shadow: 0 2px 6px rgba(0,0,0,.5); padding: 0 10px;"]]\n[[size 120%]]Remember: **The main site is for summary judgment of final work, not feedback and critique on unfinished work.**[[/size]]\n----\nIt is your responsibility to post only finished, final work. Site members are not required to justify or explain their votes.\n\nIf you understand all of the above and still wish to create this page, **[[button edit text="click here"]]** to do so.\n[[/div]]
205	**Item #:** SCP-001\n\n**Object Class:** [DATA EXPUNGED]\n\n**Special Containment Procedures:** [DATA EXPUNGED]\n\n**Description:** [DATA EXPUNGED]
206	[[>]]\n[[module Rate]]\n[[/>]]\n[[html]]\n<body>\n<style type="text/css">\nbody\n{\n   font-family:verdana,arial,helvetica,sans-serif;\n   font-size:12.8px;\n   color:#333;\n   line-height:141%\n}\nblockquote\n{\n  border:1px dashed #999;\n  padding: 0 12.8px;\n  background-color:#f4f4f4\n}\na {\n    color: #b01;\n    text-decoration: none;\n    background: transparent;\n}\n.content-panel {\n    border: solid 1px #888880;\n    border-radius: 10px;\n    background-color: #999990;\n    margin: 10px 0 15px;\n    box-shadow: 3px 3px 6px #bbb;\n    box-shadow: 0 2px 6px rgba(0,0,0,0.5), inset 0 1px rgba(255,255,255,0.3), inset 0 10px rgba(255,255,255,0.2), inset 0 10px 20px rgba(255,255,255,0.25), inset 0 -15px 30px rgba(0,0,0,0.1);\n}\n.content-panel.standalone {\n    background: #fcfdfb;\n}\n.content-panel.series {\n    padding: 0 20px;\n    margin-bottom: 20px;\n}\na.newpage {\n    color: #d61;\n    text-decoration: none;\n    background: transparent;\n}\na:hover {\n    text-decoration: underline;\n    background-color: transparent;\n}\n.collapsed {\n  display: none;\n}\n.vanished {\n  display: none;\n}\n.warning-cont {\n  color: red;\n  text-align: center;\n}\n#warning-header {\n  font-size: 25.6px;\n}\n#warning {\n  font-size: 20px;\n}\n.retrofont {\nfont-family: 'Geo', cursive;\nfont-weight: bold;\n}\n.cblwarning {\n  color: red;\n}\n.green {\n  color: green;\n}\n.smaller-g {\nwidth: 45%;\nmargin: auto;\nbackground-color: #e6e6e6;\nborder: none;\nheight: 2px;\n}\n.adden{\nfont-size: 1.3em;\n}\n/* Standard Image Block */\n.scp-image-block {\n    border: solid 1px #666;\n    box-shadow: 0 1px 6px rgba(0,0,0,.25);\n    width: 300px;\n}\n \n.scp-image-block.block-right {\n        float: right;\n    clear: right;\n    margin: 0 2em 1em 2em;\n}\n \n.scp-image-block.block-left {\n    float: left;\n    clear: left;\n    margin: 0 2em 1em 0;\n}\n \n.scp-image-block.block-center {\n    margin-right: auto;\n    margin-left: auto;\n}\n.scp-image-block img {\n    border: 0;\n    width: 300px;\n}\n.scp-image-block .scp-image-caption {\n    background-color: #eee;\n    border-top: solid 1px #666;\n    padding: 2px 0;\n    font-size: 80%;\n    font-weight: bold;\n    text-align: center;\n    width: 300px;\n}\n.scp-image-block > p {\n    margin: 0;\n}\n.scp-image-block .scp-image-caption > p {\n    margin: 0;\n    padding: 0 10px;\n    line-height: 125%\n}\n</style>\n<script type="text/javascript">\nfunction addEvent(element, eventName, callback) {\n    if (element.addEventListener) {\n        element.addEventListener(eventName, callback, false);\n    } else if (element.attachEvent) {\n        element.attachEvent("on" + eventName, callback);\n    } else {\n        element["on" + eventName] = callback;\n    }\n}\n\nwindow.toggle = function(cls) {\n  var collapsed = document.querySelectorAll("." + cls + ".collapsed");\n  var expanded = document.querySelectorAll("." + cls + ":not(.collapsed)");\n  var i;\n  for(i = 0; i < collapsed.length; i++) {\n    collapsed[i].classList.remove("collapsed");\n  }\n  for(i = 0; i < expanded.length; i++) {\n    expanded[i].classList.add("collapsed");\n  }\n};\n\n// typewriter code\nfunction Typewriter(el, str, delay) {\n  if (!(this instanceof Typewriter)) return new Typewriter(el, str, delay);\n\n  this.el = el;\n  this.str = str || el.innerHTML || el.value;\n  this.delay = delay || 100;\n  this.i = 0;\n}\n\nTypewriter.prototype.type = function() {\n  var i = this.i,\n    char = this.str.charAt(i);\n\n  if (!char) {\n    clearInterval(this.intervalID);\n    return this;\n  }\n\n  if (char === '<') this.isTag = true;\n  if (char === '>') this.isTag = false;\n\n  this.el.innerHTML += char;\n  return this.i++;\n};\n\nTypewriter.prototype.start = function() {\n  var self = this;\n  if (this.i < 0) this.i = 0;\n  if (this.el.innerHTML === this.str) this.clear();\n\n  // this.emit('start');\n  (function loop() {\n    self.type();\n    if (self.isTag) return loop();\n    self.intervalID = setTimeout(loop, self.delay);\n  }());\n\n  return this;\n};\n\nTypewriter.prototype.stop = function() {\n  this.i = -1;\n  return this;\n};\n\nTypewriter.prototype.restart = function() {\n  this.clear();\n  this.i = 0;\n  return this.start();\n};\n\nTypewriter.prototype.clear = function() {\n  this.el.innerHTML = '';\n  return this;\n};\n\nvar junkChars = ['ó','ò','ñ','ç','¿','ß','à','è','ì','ù','Œ','þ','█','%','+','=','.','«','»','£','$','€','½','¾','À','È','Ì','Ò','Ù','Â','Ê','â','ê','Ä','Ë','ä','ë','ÿ','ü','•'];\n\nvar an_iteration = "";\n\n\nvar randomIntFromInterval = function(min,max)\n{\n  return Math.floor(Math.random()*(max-min+1)+min);\n};\n\nvar getJunkChars = function(length) {\n  var res = '';\n  var i;\n  for (i = 0; i < length; i++) {\n    res += junkChars[randomIntFromInterval(0,junkChars.length - 1)];\n  }\n  return res;\n};\n\nvar selTabs = [];\n\nfunction rndiam() {\n  var rndiams = document.getElementsByClassName('rndiam');\n  var i;\n  for (i = 0; i < rndiams.length; i++) {\n    rndiams[i].innerHTML = (Math.random() * (1.56 - 0.51) + 0.51).toFixed(2);\n  }\n  setTimeout(rndiam, 2000);\n}\n\nwindow.onload = function() {\n\nselTabs = [\n  document.getElementsByClassName('selTab1')[0],\n  document.getElementsByClassName('selTab2')[0],\n  document.getElementsByClassName('selTab3')[0],\n  document.getElementsByClassName('selTab4')[0],\n  document.getElementsByClassName('selTab5')[0]\n];\n\nvar msg1 = '= SCP-001 | Technical Clearance Required =';\n\nvar delay1 = 50;\nvar delay2 = 25;\n\nvar toD1 = delay1 * (msg1.length);\n\nvar stopEarly = function(){\n  document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished");\n};\n\nvar typ1 =Typewriter(document.querySelectorAll('#warning-header')[0],msg1,delay1)\n\ntyp1.start();\nsetTimeout(function() { document.getElementById("warning").classList.remove("vanished"); }, toD1 + 500);\nsetTimeout(function () { document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished"); }, toD1 + 1500);\n\n  rndiam();\n\n  var today = new Date();\n  var todayformat = "" + (today.getFullYear() - 1) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today").innerHTML = todayformat;\n  var todayformat2 = "" + (today.getFullYear() - 0) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today2").innerHTML = todayformat2;\n  document.getElementById("yearplus3").innerHTML = "" + (today.getFullYear() + 3);\n\n  an_iteration = document.getElementById("part1").innerHTML;\n};\n\nwindow.new_iteration = function() {\n  var new_iters = document.getElementsByClassName("newiter");\n  console.log(new_iters);\n  new_iters[new_iters.length - 1].classList.add("vanished");\n  var inner = document.getElementsByClassName("inner");\n  inner[inner.length - 1].innerHTML = "<hr /><hr />" + an_iteration;\n};\n</script>\n</script>\n<div class="warning-cont">\n<h2 id="warning-header">\n<p>\n</p>\n<h3 id="warning" class="vanished">  \n<p>This document exists as technical containment for an anomaly and thus does not adhere to standard SCiPNET formatting. Input 5/TECHNICAL clearance to proceed.\n</p>\n</div>\n\n<div id="collapsible_to_open_skip" class="article vanished">\n<p><center><a onclick="toggle('article');">[ INPUT PERSONAL IDENTIFICATION NUMBER ]</a></center></p>\n</div>\n<div class="article collapsed">\n <p><center><a onclick="toggle('article');">[ PERSONAL IDENTIFICATION NUMBER VERIFIED ]</a></center></p>\n<div id="part1">\n<hr />\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://topia.wikidot.com/local--files/jamcon-001/sky.jpg" style="width:300px;" alt="destruction.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>The sky above Point Alpha during a breach of SCP-001.</p>\n</div>\n</div>\n<p><b>Item #:</b> SCP-001</p>\n<p><b>Object Class:</b> Ontokinetic</p>\n<p><b>Containment Class:</b> Keter</p>\n<p><b>Special Containment Procedures:</b> If the sky begins changing color with no prior stimuli or indication, all Foundation sites are to go onto ALERT LEVEL 7 immediately. Personnel at Site-05 and -06 are to enter ALERT LEVEL 8 and begin evaluating new methods of containing SCP-001.</p>\n<p>Site-05 has been constructed around Point Alpha. Procedure 001-ENTRY is in effect to contain SCP-001-1. A monitor containing Document 032, as well as a speaker reading the document, are to be placed at least five meters away from SCP-001-1. If SCP-001-1 swells up to 2 meters or more in diameter, the speed of the narration is to increase. This system is to have a direct cable link to the Foundation SCiPNET database at Site-06, and several redundant power generators are to ensure this system stays operational at all times.</p>\n<p><b>Description:</b> SCP-001 is an ongoing ZK-Class "Reality Failure" Scenario. SCP-001 is prevented from happening through several procedures manipulating its sub-anomalies. The only known observable symptom of SCP-001 is the sky becoming a different color; it is believed that, if SCP-001 were to progress to the point where other symptoms began to occur, it would become irreversible.</p>\n<p>SCP-001-1 is a levitating object contained within Point Alpha, a cave chamber five kilometers south of the ruins of Babylon. SCP-001-1 visually resembles a perfectly smooth sphere with a texture similar to blurred television static. Measurement instruments indicate SCP-001-1 is currently <span class="rndiam"></span> meters in diameter. Solids and liquids that pass within the bounds of SCP-001-1 are replaced entirely with argon gas, giving the illusion of disappearing.</p>\n<p>SCP-001-2 is an entity that is capable of being seen within Point Alpha. SCP-001-2's manifestations are random; witnesses report seeing SCP-001-2 "past the walls of the cave." From descriptions of eyewitnesses, SCP-001-2 is humanoid in shape and large in stature, and is only visible when looking through SCP-001-1. Descriptions of SCP-001-2 vary from person to person; common features include an emaciated figure, long limbs, and prominent facial features. SCP-001-2 takes the position of lying against the wall. It is most often seen with a neutral expression; however, the expression occasionally changes to discomfort or bliss.</p>\n<p>It has been determined from historical precedent that "entertaining" SCP-001-2 through SCP-001-1 will prevent SCP-001.</p>\n<div class="scp-image-block block-left" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/cave.jpg" style="width:300px;" alt="cave.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Point Alpha shortly after it was acquired by the Foundation, with Agents Samuel and Boise preforming preliminary evaluation. SCP-001-1 not visible.</p>\n</div>\n</div>\n<p><b>History:</b> Point Alpha and its contents are believed to date back to prehistory. Babylonian scripts indicate that SCP-001-1 was routinely patrolled by a group of sages, known as the "Order of White and Black," who refused passage to anyone who could demonstrate the ability to read or write.</p>\n<p>This group was made defunct by the rise of Achaemenid Persia, which contained SCP-001-1 with a group of Zoroastrian monks. Very few accounts of SCP-001-1 are present; recovered accounts assert that all information about SCP-001-1 and its significance was passed orally.</p>\n<p>The most substantial account of SCP-001-1 and SCP-001-2 at this time dates back to the Greek philosopher Xera, who made an expedition into the Achaemenid Empire and found Point Alpha. During Alexander of Macedon's conquest of the Achaemenid Empire, he took interest in Xera's texts and continued the Achaemenid containment of SCP-001-1. However, this new institution, known as the "Cronus Guard", was given Greek epics to read in order to "punish Cronus" for his actions. This institution lasted through the partition of Macedon into the Seleucid Empire, the rise of the Sassanid Empire, and even the formation of the Rashidun Caliphate.</p>\n<p>The Cronus Guard were eventually replaced with an Islamic equivalent: "The Society for the Containment of the Babel Demon." This group was the first to preform substantial research on SCP-001-1, and the first to accurately link SCP-001 to SCP-001-1.</p>\n<div class="socotbd">\n<p style="font-size: 82%"><a onclick="toggle('socotbd');">View Attachment: Excerpts from the notes of the Society for the Containment of the Babel Demon</a></p>\n</div>\n<div class="socotbd collapsed">\n <p style="font-size: 82%"><a onclick="toggle('socotbd');">\nClose Attachment</a></p>\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/book.jpg" style="width:300px;" alt="book.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Records from the Society for the Containment of the Babel Demon, recovered in the early 19<sup>th</sup> century within ORIA archives.</p>\n</div>\n</div>\n<blockquote>\n<p><b>Literature Read:</b> The first quatrain of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON squirms but otherwise does not react. Consistent with reading of Homer's Epics by the Chronus Guard.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> The remaining quatrains of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> No changes from observed behavior.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> A poem written by Guardian Muhammad ibn Buya'aa meant explicitly for DEMON</p>\n<p><b>Result:</b> DEMON thrashes more than usual until the cessation of the reading. ORB begins to expand until the <i>Genealogies of the Nobles</i> is read to DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> None, for 10 days</p>\n<p><b>Result:</b> ORB is observed to expand, and DEMON is observed to thrash. Externally, the sky above the camp is noted to turn a darker red color, until the <i>Rubaiyat of Omar Khayyam</i> is read again.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON thrashes more than during previous readings of the <i>Rubaiyat of Omar Khayyam</i>. It is believed that the Quran may no longer have any effect on DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>One Thousand and One Nights</i></p>\n<p><b>Result:</b> DEMON ceases thrashing. New literature may be necessary to prevent further phenomena from occurring.</p>\n</blockquote>\n</div>\n<p>The Society was eventually absorbed into the Office for the Reclamation of Islamic Artifacts, who assumed containment of SCP-001-1. After Incident 001-EXAL, the Foundation purchased Point Alpha from ORIA for a large sum of currency and several Safe-class anomalies.</p>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Incident 001-EXAL</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<p style="font-size: 82%"><i>Note: Some documentation taken from ORIA's account of Incident 001-EXAL.</i></p>\n<p>On <span id="today"></span>, personnel with ORIA read the book <i>Tafsir al-Ahlam al-kabir</i>, or "Great Book of Interpretation of Dreams" as per normal containment of SCP-001-1. However, SCP-001-2 was observed to rapidly thrash while screaming. SCP-001-1 itself swelled from 50.3 centimeters to 3.2 meters in diameter.</p>\n<p>Shortly following this event, SCP-001 began to worsen. The sky worldwide turned a black/white color similar to the texture of SCP-001-1. In addition, reality bending phenomena began worldwide, causing deformed geography, the manifestation of dangerous anomalous objects and several natural disasters. This event was ended after ORIA personnel read SCP-001-1 an undisclosed number of as-of-yet unread books, which caused SCP-001-2 to stop thrashing and SCP-001 to restore to its pre-incident point. However, the damage caused by this incident was deemed enough to require an activation of <a target="_top" href="http://www.scp-wiki.net/scp-2000">SCP-2000</a>.</p>\n<p>This prompted the Foundation to take control of containment of SCP-001. See <b>History</b> segment above for more information.</p>\n<div class="info">\n<p style="font-size: 82%"><a onclick="toggle('info');">Input Level 5/001 Credentials</a></p>\n</div>\n<div class="info collapsed">\n <p style="font-size: 82%"><a onclick="toggle('info');">\nAccess Granted</a></p>\n<p>During Incident 001-Alpha, vocalizations in <a target="_top" href="http://www.scp-wiki.net/scp-140">Daevish</a> were recorded by ORIA instruments within Point Alpha. The source is presently unknown. The following are approximate translations of these vocalizations.</p>\n<blockquote>\n<p>It has passed some [space/time].</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. It [has had/used to be] fun. It is [time/space] to leave.</p>\n</blockquote>\n<blockquote>\n<p>You cannot [stay/root/hold] yourself forever.</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. You [must/will] awaken.</p>\n</blockquote>\n<blockquote>\n<p>No, you have [entered/reentered] slumber for too long. Wake up.</p>\n</blockquote>\n<blockquote>\n<p>[King/Prince/loved one], it is time for you to wake up.</p>\n</blockquote>\n<blockquote>\n<p>It is fun to be in the [UNKNOWN: gaera] but you cannot be [in/rooted] there [forever/unending]. It is [entertaining/deathlike], but you must wake up.</p>\n</blockquote>\n<blockquote>\n<p>Wake up, [King/Prince/loved one]. We miss you.</p>\n</blockquote>\n</div>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Addenda</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<div class="discus">\n<p style="font-size: 82%"><a onclick="toggle('discus');">Input Level 5/001 Clearance</a></p>\n</div>\n<div class="discus collapsed">\n <p style="font-size: 82%"><a onclick="toggle('discus');">\nAccess Granted</a></p>\n<div div class="content-panel standalone series">\n<center>\n<h2>Discussion Thread 001-398:<br />Continued Containment</h2>\n<p style="font-size: 82%">Started on: <span id="today2"></span></p>\n<p style="font-size: 82%">Started by: <tt>HMCL Robinson</tt></p>\n</center>\n</div>\n<blockquote>\n<p><b><tt>HMCL Robinson:</tt></b> The usage of this thread will be for containment directives for SCP-001. In the past year that we have spent containing it, SCP-001-1's literature needs have become increasingly draconic: we can very rarely reread books without it beginning to thrash, and even then we're running out of high-quality literature to give it. We need a more long-term, reliable solution for containment. SCP-001's file has been temporarily opened up to all personnel with Level 2 clearance. Anybody can submit an idea.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Use Artificial Intelligence Constructs to automatically generate new stories for SCP-001-2.</p>\n<p><b>Status: <tt>[ <span class="green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> TSATPWTCOTTTADC.aic was able to generate 10,000 volumes of stories imitating Greek literature. However, when the first was read to SCP-001-1, it began thrashing beyond acceptable measures until it was read approved reading. No further computer-generated works are to be given to SCP-001-1.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Hire a full team of authors to create literature for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> In the past, literature written specifically for SCP-001-1 have failed to contain it (see: SCtBD Document #249). With SCP-001-2's current state this is not to be attempted again.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> This could represent a possible information leak. Test denied.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Reason:</b> With SCP-001-2's recent containment breach, these measures are approved.</p>\n<p><b>Result:</b> SCP-001-2 observed to stop thrashing and stay completely still, seemingly smiling. In addition, SCP-███ was able to be read for 14 consecutive readings before SCP-001-2 resumed normal activity.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Regularly read mainlist documentation for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Replacement of the SCP-001 file with several "001 Proposals" with much more grand implications than normal SCP files.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> By only occasionally reading 001 files, SCP-001-2 is observed to be more calm on average. The previously observed actions of retaliation in response to specially crafted literature do not apply here.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Given the variation of the current 2957 SCP mainlist files, SCP-001-1 is to be read a random arrangement of these files, in repeat.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> This strategy succeeded for approximately 3 years, allowing for several repeats. However, on <span id="yearplus3"></span>-9-30, SCP-001-2 began thrashing rapidly, requiring newer documentation to sedate.</p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-4999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> SCP-001-1 has required progressively more SCP mainlist articles for containment, even to the point of the fabrication of some anomalous entities.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Rewriting of several older SCP mainlist files to be more exaggerated and narrative-like.</p>\n<p><b>Status: <tt>[ <span style="color: orange">TENTATIVELY APPROVED</span> ]</tt></b></p>\n<p><b>Note:</b> <i>Deliberately exaggerating our files for the purpose of this entity is crossing some lines, but for now, it will have to do. However, we need a better way. I'm calling a committee. We need to put a stop to this.</i></p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Creation of a file describing SCP-001 that uses recursion to effectively be "never-ending." Psychoanalysis related to SCP-001-2's apparent "enjoyment" of works implies a slight ego and bias towards works about itself, and past strategies with this idea have worked for long periods of time. File is to be archived as a "technical" file.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> Procedure 001-ENTRY implemented. [DATA EXPUNGED]</p>\n</blockquote>\n</div>\n\n<div class="newiter">\n<p style="font-size: 82%"><a onclick="new_iteration();">Input Level 5/TECHNICAL Clearance</a></p>\n</div>\n<div class="inner">\n<!-- :-) -->\n</div>\n</div>\n</body>\n[[/html]]
207	[!-- NOTICE\n\nDO NOT EDIT THIS PAGE UNLESS YOU ARE POSTING A 001 PROPOSAL.\n\nPLEASE POST YOUR LINK AT THE BOTTOM OF THE LIST OF PROPOSALS. DO NOT CHANGE THE ORDER OF THE LIST.\n\nPROPER PROPOSAL LINK FORMAT IS AS FOLLOWS:\n\n[[[username-s-proposal| CODE NAME: username]]] - Title\n\nDO NOT EDIT OR ATTEMPT TO CHANGE MEMETIC HAZARD IMAGE.\n\nDOING SO WILL BE CONSIDERED VANDALISM, AND IS A BANNABLE OFFENSE. --]\n\n[[=]]\n++ THE FOLLOWING FILES HAVE BEEN CLASSIFIED\n\n+ [[size 200%]]TOP SECRET[[/size]]\n\n++ BY ORDER OF THE ADMINISTRATOR\n[[/=]]\n-----\n[[div class="content-panel standalone series"]]\n[[==]]\n**GENERAL NOTICE 001-Alpha:** In order to prevent knowledge of SCP-001 from being leaked, several/no false SCP-001 files have been created alongside the true file/files. All files concerning the nature of SCP-001, including the decoy/decoys, are protected by a memetic kill agent designed to immediately cause cardiac arrest in any nonauthorized personnel attempting to access the file. Revealing the true nature/natures of SCP-001 to the general public is cause for execution, except as required under ████-███-██████.\n[[/==]]\n[[/div]]\n-----\n[[=]]\n+ [[size 200%]]WARNING:[[/size]]\n\nANY NON-AUTHORIZED PERSONNEL ACCESSING THIS FILE WILL BE IMMEDIATELY TERMINATED THROUGH **BERRYMAN-LANGFORD** MEMETIC KILL AGENT. SCROLLING DOWN WITHOUT PROPER MEMETIC INOCULATION WILL RESULT IN IMMEDIATE CARDIAC ARREST FOLLOWED BY DEATH.\n\n+ YOU HAVE BEEN WARNED.\n[[/=]]\n-----\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n@@@@\n[[=image fractal001 style="width: 500px; border: solid 1px #000000; padding: 1px"]]\n[[=]]\n**MEMETIC KILL AGENT ACTIVATED**\n\n**CONTINUED LIFE SIGNS CONFIRMED**\n\n**REMOVING SAFETY INTERLOCKS**\n\n//Welcome, authorized personnel. Please select your desired file.//\n[[/=]]\n-----\n[[div class="content-panel standalone series"]]\n\n[[=]]\n[[[Jonathan Ball's Proposal|CODE NAME: Jonathan Ball]]] - Sheaf of Papers\n\n[[[Dr. Gears's Proposal|CODE NAME: Dr. Gears]]] - The Prototype\n\n[[[Dr. Clef's Proposal|CODE NAME: Dr. Clef]]] - The Gate Guardian\n\n[[[qntm's Proposal|CODE NAME: qntm]]] - The Lock\n\n[[[scp-001-o5 |CODE NAME: Bright]]] - The Factory\n\n[[[dr-manns-proposal|CODE NAME: Dr. Mann]]] - The Spiral Path\n\n[[[mackenzie-s-proposal|CODE NAME: Dr. Mackenzie]]] - The Legacy\n\n[[[sandrewswann-s-proposal|CODE NAME: S. Andrew Swann]]] - The Database\n\n[[[scantron-s-proposal| CODE NAME: Scantron]]] - The Foundation\n\n[[[djoric-dmatix-proposal| CODE NAME: Djoric/Dmatix]]] - Thirty-Six\n\n[[[roget-s-proposal| CODE NAME: Roget]]] - Keter Duty\n\n[[[Ouroboros| CODE NAME: djkaktus/TwistedGears]]] - Ouroboros\n\n[[[Kate McTiriss's Proposal| CODE NAME: Kate McTiriss]]] - A Record\n\n[[[kalinins proposal| CODE NAME: Kalinin]]] - Past and Future\n\n[[[wrong proposal| CODE NAME: Wrong]]] - The Consensus\n\n[[[shaggydredlocks-proposal| CODE NAME: S. D. Locke]]] - When Day Breaks\n\n[[[spikebrennan-s-proposal| CODE NAME: Spike Brennan]]] - God's Blind Spot\n\n[[[WJS Proposal| CODE NAME: WJS]]] - Normalcy\n\n[[[billiths-proposal | CODE NAME: BILLITH]]] - The World at Large\n\n[[[tanhony-s-proposal | CODE NAME: Tanhony]]] - Dead Men\n\n[[[lily-s-proposal | CODE NAME: Lily]]] - The World's Gone Beautiful\n\n[[[tuftos-proposal | CODE NAME: Tufto]]] - The Scarlet King\n\n[[[jim-north-s-proposal | CODE NAME: Jim North]]] - A Simple Toymaker\n\n[[[i-h-p-proposal | CODE NAME: I.H. Pickman]]] - Story of Your Life\n\n[[[scp-001-ex | CODE NAME: The Great Hippo (feat. PeppersGhost)]]] - A Good Boy\n\n[[[wmdd-s-proposal | CODE NAME: weizhong|thedeadlymoose|Drewbear|Dexanote]]] - Project Palisade\n\n[[[captain-kirby-s-proposal | CODE NAME: Captain Kirby]]] - O5-13\n\n[[[pedantique-s-proposal | CODE NAME: Pedantique]]] - Fishhook\n\n[[[not-a-seagull-proposal | CODE NAME: not_a_seagull]]] - The Sky above the Port\n\n[[[jack-ike-s-proposal-i| CODE NAME: Meta Ike]]] - The Solution\n\n[[[jack-ike-s-proposal-ii| CODE NAME: Noir Box]]] - Tindalos Trinity\n\n[[[tanhony-s-proposal-ii | CODE NAME: Tanhony II]]] - The Black Moon\n[[/=]]\n[[/div]]
208	[[>]]\n[[module Rate]]\n[[/>]]\n\n**Item #:** SCP-001\n\n**Object Class:** [DATA EXPUNGED]\n\n**Special Containment Procedures:** [DATA EXPUNGED]\n\n**Description:** [DATA EXPUNGED]
209	[[module ListPages category="*" order="random" limit="1" tag="scp"]]\n[[include :snippets:redirect url=%%link%%]]\n[[/module]]
210	[[module ListPages category="*" order="random" limit="1" tag="tale"]]\n[[include :snippets:redirect url=%%link%%]]\n[[/module]]
283	[[[all-wikis | list all wikis]]]\n\n>hello\n\n> hello
288	[[module ForumCategory]]\n\n[!-- please do not alter this page if you want to keep your forum working --]
211	[[module CSS]]\n@import url(http://scp-wiki.wdfiles.com/local--code/info%3Astyle/1);\n[[/module]]\n\n[[div class="creditRate"]]\n[[div class="rateBox" [[iftags heritage]]style="display:none;"[[/iftags]]]]\n[[div class="rate-box-with-credit-button"]]\n[[module Rate]]\n[[div class="creditButton"]]\n[[a href="#u-credit-view" class="fa fa-info"]]@@@@[[/a]]\n[[/div]]\n[[/div]]\n[[/div]]\n[[div class="rateBox" [[iftags -heritage]]style="display:none;"[[/iftags]]]]\n[[div class="heritage-rating-module"]]\n[[div class="heritage-emblem"]]\n[[image http://scp-wiki.wdfiles.com/local--files/component:heritage-rating/scp-heritage-v3.png link="heritage-collection" style="max-width: none;"]]\n[[/div]]\n[[module Rate]]\n[[div class="creditButton"]]\n[[a href="#u-credit-view" class="fa fa-info"]]@@@@[[/a]]\n[[/div]]\n[[/div]]\n[[/div]]\n[[/div]]\n\n[[div style="clear:both;"]]\n[[/div]]\n\n[[div id="credit-view"]]\n[[div class="fader"]]\n[[iframe http://scp-jp-sandbox3.wdfiles.com/local--code/credit%3Abackmodule/1 ]]\n[[/div]]\n\n[[div class="modalcontainer"]]\n[[div class="modalbox"]]\n[[=]]\n++* Info\n[[/=]]\n------\n[[div class="credit first"]]\n[[iframe http://scp-wiki.wdfiles.com/local--code/info%3Astyle/3 class="close-credits" style="height:2em;margin: 0;padding: 0;border: 0;background: transparent;"]]
212	[[/div]]\n------\n[[div class="credit" style="text-align:center; margin-top: 0.5em;"]]\n[[a href="#u-credit-otherwise"]]More information[[/a]]\n[[/div]]\n[!--\n\n[[=]]\n[[div [[iftags heritage]]style="display:block;position:relative;height:30px;margin-right:30px;"[[/iftags]] class="Dendo"]]\n[[div class="heritage-rating-module"]]\n[[div class="heritage-emblem"]]\n[[image http://scp-wiki.wdfiles.com/local--files/component:heritage-rating/scp-heritage-v3.png link="heritage-collection" style=" max-width: none;"]]\n[[/div]]\n[[module Rate]]\n[[/div]]\n[[/div]]\n[[div [[iftags heritage]]style="display:none;"[[/iftags]]]]\n[[module Rate]]\n[[/div]]\n[[/=]]\n\n--]\n[[/div]]\n[[/div]]\n[[/div]]\n[[div id="credit-otherwise"]]\n[[div class="fader"]]\n[[iframe http://scp-jp-sandbox3.wdfiles.com/local--code/credit%3Abackmodule/3 ]]\n[[/div]]\n[[div class="modalcontainer"]]\n[[div class="modalbox"]]\n[[=]]\n++* More information\n[[/=]]\n------\n[[div class="credit otherwise"]]
213	[[/div]]\n------\n[[div class="credit-back" style="text-align: center;"]]\n[[iframe http://scp-wiki.wdfiles.com/local--code/info%3Astyle/2 style="height:2em;width: 100%;margin: 0;padding: 0;border: 0;background: transparent;" scrolling="no"]]\n[[/div]]\n[[div class="creditBottomRate" style="position:relative;height:30px;"]]\n[[div style="text-align: center; top: 0px;"]]\n[[div [[iftags heritage]]style="display:block;"[[/iftags]] class="Dendo"]]\n[[div class="heritage-rating-module"]]\n[[div class="heritage-emblem"]]\n[[image http://scp-wiki.wdfiles.com/local--files/component:heritage-rating/scp-heritage-v3.png link="heritage-collection" style=" max-width: none;"]]\n[[/div]]\n[[module Rate]]\n[[/div]]\n[[/div]]\n[[div [[iftags heritage]]style="display:none;"[[/iftags]]]]\n[[module Rate]]\n[[/div]]\n[[/div]]\n[[/div]]\n[[/div]]\n[[/div]]\n[[/div]]
214	A profile has not been set up for this user.
215	A profile has not been set up for this user.
222	[[module ManageSite]]
275	+ Members:\n\n[[module Members]]\n\n+ Moderators\n\n[[module Members group="moderators"]]\n\n+ Admins\n\n[[module Members group="admins"]]
276	[[div style="float:right; width: 50%;"]]\n[[module TagCloud limit="200" target="system:page-tags"]]\n[[/div]]\n[[module PagesByTag]]\n\n[!--\n\nYou can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module \nBut if you want to keep the tag functionality working - do not remove these modules.\n\n--]
277	[[module Pages preview="true"]]
278	No profile has been set up yet for this user.
284	[[[all-wikis | list all wikis]]]\n\n>hello\n\n> hello\n\n>>hello\n\n>> hello
289	[[module ForumThread]]\n\n[!-- please do not alter this page if you want to keep your forum working --]
290	[[module ForumNewThread]]\n\n[!-- please do not alter this page if you want to keep your forum working --]
291	[[module RecentPosts]]\n\n[!-- please do not alter this page if you want to keep your forum working --]
292	[[[all-wikis | list all wikis]]]\n\n>hello\n\n> hello\n\n>>hello\n\n>> hello\n\n>  hello\n\none \\\ntwo\n\n[[module Comments]]\n\n[[module654 Comments]]
293	[[[all-wikis | list all wikis]]]\n\n>hello\n\n> hello\n\n>>hello\n\n>> hello\n\n>  hello\n\none \\\ntwo\n\n[[module Comments]]\n\n[[module654 Comments]]\n\n[[[all-wikis | list all|wikis]]
294	[[[all-wikis | list all wikis]]]\n\n>hello\n\n> hello\n\n>>hello\n\n>> hello\n\n>  hello\n\none \\\ntwo\n\n[[module Comments]]\n\n[[module654 Comments]]\n\n[[[all-wikis | list all|wikis]]\n\n-----
230	[[module CSS]]\n#page-title { color:orange; }\n[[/module]]\n\n[!--\nWelcome to your MyWikidot Custom Installation!\n--]\nCongratulations, you have successfully configured and launched your Wikidot custom installation!\n+ What to do next\n++ Experienced Wikidot users should [[[mywikidot-info|start here]]].\n++ Customize this wiki\nWikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n++ Customize the default templates\nThere are 2 initial default templates for other wikis. One is located at [[[template-en::start|template-en]]] and the other at [[[template-blog::start|template-blog]]]. If someone creates a new wiki, these are presented as choices and the selected template is cloned to the new wiki's address. You should customize [[[template-en::start|template-en]]] and [[[template-blog::start|template-blog]]] to suit your needs.\n++ Create more templates\nSimply create new wikis with **web site names** starting with "template-" (e.g. "template-pl", "template-recipes") and your users will be have even more choices for the basic wiki layout they want to start with. \n++ Visit Wikidot.org\nGo to **[*http://www.wikidot.org www.wikidot.org]** -- home of the Wikidot open source software -- for extra documentation, howtos, tips and support.\n++ Visit the Wikidot Community Site\nGo to **[*http://community.wikidot.com community.wikidot.com]** -- for even more tips, tricks and help from a very active community of Wikidot users.\n++ Visit the MyWikidot.local Project Site\nGo to **[*http://my-wd-local.wikidot.com/ my-wd-local.wikidot.com]** -- for tips, discussions and how-to articles.\n---------------\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org] and the developers discussion at [*http://groups.google.com/group/wikidot Wikidot dev-list].\n+ Search all wikis\n[[module SearchAll]]\n+ Search users\n[[module SearchUsers]]
279	[[[all-wikis | list all wikis]]]
285	[[[all-wikis | list all wikis]]]\n\n>hello\n\n> hello\n\n>>hello\n\n>> hello\n\none \\\ntwo
237	[[module CSS disable="true"]]\n#page-title { color:orange; }\n[[/module]]\n\n[!--\nWelcome to your MyWikidot Custom Installation!\n--]\nCongratulations, you have successfully configured and launched your Wikidot custom installation!\n+ What to do next\n++ Experienced Wikidot users should [[[mywikidot-info|start here]]].\n++ Customize this wiki\nWikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n++ Customize the default templates\nThere are 2 initial default templates for other wikis. One is located at [[[template-en::start|template-en]]] and the other at [[[template-blog::start|template-blog]]]. If someone creates a new wiki, these are presented as choices and the selected template is cloned to the new wiki's address. You should customize [[[template-en::start|template-en]]] and [[[template-blog::start|template-blog]]] to suit your needs.\n++ Create more templates\nSimply create new wikis with **web site names** starting with "template-" (e.g. "template-pl", "template-recipes") and your users will be have even more choices for the basic wiki layout they want to start with. \n++ Visit Wikidot.org\nGo to **[*http://www.wikidot.org www.wikidot.org]** -- home of the Wikidot open source software -- for extra documentation, howtos, tips and support.\n++ Visit the Wikidot Community Site\nGo to **[*http://community.wikidot.com community.wikidot.com]** -- for even more tips, tricks and help from a very active community of Wikidot users.\n++ Visit the MyWikidot.local Project Site\nGo to **[*http://my-wd-local.wikidot.com/ my-wd-local.wikidot.com]** -- for tips, discussions and how-to articles.\n---------------\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org] and the developers discussion at [*http://groups.google.com/group/wikidot Wikidot dev-list].\n+ Search all wikis\n[[module SearchAll]]\n+ Search users\n[[module SearchUsers]]
286	[[[all-wikis | list all wikis]]]\n\n>hello\n\n> hello\n\n>>hello\n\n>> hello\n\none \\\ntwo\n\n[[module Comments]]\n\n[[module654 Comments]]
238	[[module CSS]]\n#page-title { color:green; }\n[[/module]]\n\n[!--\nWelcome to your MyWikidot Custom Installation!\n--]\nCongratulations, you have successfully configured and launched your Wikidot custom installation!\n+ What to do next\n++ Experienced Wikidot users should [[[mywikidot-info|start here]]].\n++ Customize this wiki\nWikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n++ Customize the default templates\nThere are 2 initial default templates for other wikis. One is located at [[[template-en::start|template-en]]] and the other at [[[template-blog::start|template-blog]]]. If someone creates a new wiki, these are presented as choices and the selected template is cloned to the new wiki's address. You should customize [[[template-en::start|template-en]]] and [[[template-blog::start|template-blog]]] to suit your needs.\n++ Create more templates\nSimply create new wikis with **web site names** starting with "template-" (e.g. "template-pl", "template-recipes") and your users will be have even more choices for the basic wiki layout they want to start with. \n++ Visit Wikidot.org\nGo to **[*http://www.wikidot.org www.wikidot.org]** -- home of the Wikidot open source software -- for extra documentation, howtos, tips and support.\n++ Visit the Wikidot Community Site\nGo to **[*http://community.wikidot.com community.wikidot.com]** -- for even more tips, tricks and help from a very active community of Wikidot users.\n++ Visit the MyWikidot.local Project Site\nGo to **[*http://my-wd-local.wikidot.com/ my-wd-local.wikidot.com]** -- for tips, discussions and how-to articles.\n---------------\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org] and the developers discussion at [*http://groups.google.com/group/wikidot Wikidot dev-list].\n+ Search all wikis\n[[module SearchAll]]\n+ Search users\n[[module SearchUsers]]
239	[!--\nWelcome to your MyWikidot Custom Installation!\n--]\nCongratulations, you have successfully configured and launched your Wikidot custom installation!\n+ What to do next\n++ Experienced Wikidot users should [[[mywikidot-info|start here]]].\n++ Customize this wiki\nWikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n++ Customize the default templates\nThere are 2 initial default templates for other wikis. One is located at [[[template-en::start|template-en]]] and the other at [[[template-blog::start|template-blog]]]. If someone creates a new wiki, these are presented as choices and the selected template is cloned to the new wiki's address. You should customize [[[template-en::start|template-en]]] and [[[template-blog::start|template-blog]]] to suit your needs.\n++ Create more templates\nSimply create new wikis with **web site names** starting with "template-" (e.g. "template-pl", "template-recipes") and your users will be have even more choices for the basic wiki layout they want to start with. \n++ Visit Wikidot.org\nGo to **[*http://www.wikidot.org www.wikidot.org]** -- home of the Wikidot open source software -- for extra documentation, howtos, tips and support.\n++ Visit the Wikidot Community Site\nGo to **[*http://community.wikidot.com community.wikidot.com]** -- for even more tips, tricks and help from a very active community of Wikidot users.\n++ Visit the MyWikidot.local Project Site\nGo to **[*http://my-wd-local.wikidot.com/ my-wd-local.wikidot.com]** -- for tips, discussions and how-to articles.\n---------------\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org] and the developers discussion at [*http://groups.google.com/group/wikidot Wikidot dev-list].\n+ Search all wikis\n[[module SearchAll]]\n+ Search users\n[[module SearchUsers]]
240	[[module CSS]]\n#page-title { color: purple; }\n[[/module]]\n\n[!--\nWelcome to your MyWikidot Custom Installation!\n--]\nCongratulations, you have successfully configured and launched your Wikidot custom installation!\n+ What to do next\n++ Experienced Wikidot users should [[[mywikidot-info|start here]]].\n++ Customize this wiki\nWikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n++ Customize the default templates\nThere are 2 initial default templates for other wikis. One is located at [[[template-en::start|template-en]]] and the other at [[[template-blog::start|template-blog]]]. If someone creates a new wiki, these are presented as choices and the selected template is cloned to the new wiki's address. You should customize [[[template-en::start|template-en]]] and [[[template-blog::start|template-blog]]] to suit your needs.\n++ Create more templates\nSimply create new wikis with **web site names** starting with "template-" (e.g. "template-pl", "template-recipes") and your users will be have even more choices for the basic wiki layout they want to start with. \n++ Visit Wikidot.org\nGo to **[*http://www.wikidot.org www.wikidot.org]** -- home of the Wikidot open source software -- for extra documentation, howtos, tips and support.\n++ Visit the Wikidot Community Site\nGo to **[*http://community.wikidot.com community.wikidot.com]** -- for even more tips, tricks and help from a very active community of Wikidot users.\n++ Visit the MyWikidot.local Project Site\nGo to **[*http://my-wd-local.wikidot.com/ my-wd-local.wikidot.com]** -- for tips, discussions and how-to articles.\n---------------\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org] and the developers discussion at [*http://groups.google.com/group/wikidot Wikidot dev-list].\n+ Search all wikis\n[[module SearchAll]]\n+ Search users\n[[module SearchUsers]]
241	[[module CSS]]\n#page-title { color: black; }\n[[/module]]\n\n[!--\nWelcome to your MyWikidot Custom Installation!\n--]\nCongratulations, you have successfully configured and launched your Wikidot custom installation!\n+ What to do next\n++ Experienced Wikidot users should [[[mywikidot-info|start here]]].\n++ Customize this wiki\nWikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n++ Customize the default templates\nThere are 2 initial default templates for other wikis. One is located at [[[template-en::start|template-en]]] and the other at [[[template-blog::start|template-blog]]]. If someone creates a new wiki, these are presented as choices and the selected template is cloned to the new wiki's address. You should customize [[[template-en::start|template-en]]] and [[[template-blog::start|template-blog]]] to suit your needs.\n++ Create more templates\nSimply create new wikis with **web site names** starting with "template-" (e.g. "template-pl", "template-recipes") and your users will be have even more choices for the basic wiki layout they want to start with. \n++ Visit Wikidot.org\nGo to **[*http://www.wikidot.org www.wikidot.org]** -- home of the Wikidot open source software -- for extra documentation, howtos, tips and support.\n++ Visit the Wikidot Community Site\nGo to **[*http://community.wikidot.com community.wikidot.com]** -- for even more tips, tricks and help from a very active community of Wikidot users.\n++ Visit the MyWikidot.local Project Site\nGo to **[*http://my-wd-local.wikidot.com/ my-wd-local.wikidot.com]** -- for tips, discussions and how-to articles.\n---------------\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org] and the developers discussion at [*http://groups.google.com/group/wikidot Wikidot dev-list].\n+ Search all wikis\n[[module SearchAll]]\n+ Search users\n[[module SearchUsers]]
280	++ Top Sites\n[[module MostActiveSites]]\n\n++ Recent edits (all websites)\n[[module RecentWRevisions]]\n\n++ Top Forums\n[[module MostActiveForums]]\n\n++ Top Sites\n[[module SiteGrid2 limit="20"]]\n\n++ New users\n[[module NewWUsers limit="50"]]
242	[[module CSS]]\n#page-title { color: green; }\n[[/module]]\n\n[!--\nWelcome to your MyWikidot Custom Installation!\n--]\nCongratulations, you have successfully configured and launched your Wikidot custom installation!\n+ What to do next\n++ Experienced Wikidot users should [[[mywikidot-info|start here]]].\n++ Customize this wiki\nWikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n++ Customize the default templates\nThere are 2 initial default templates for other wikis. One is located at [[[template-en::start|template-en]]] and the other at [[[template-blog::start|template-blog]]]. If someone creates a new wiki, these are presented as choices and the selected template is cloned to the new wiki's address. You should customize [[[template-en::start|template-en]]] and [[[template-blog::start|template-blog]]] to suit your needs.\n++ Create more templates\nSimply create new wikis with **web site names** starting with "template-" (e.g. "template-pl", "template-recipes") and your users will be have even more choices for the basic wiki layout they want to start with. \n++ Visit Wikidot.org\nGo to **[*http://www.wikidot.org www.wikidot.org]** -- home of the Wikidot open source software -- for extra documentation, howtos, tips and support.\n++ Visit the Wikidot Community Site\nGo to **[*http://community.wikidot.com community.wikidot.com]** -- for even more tips, tricks and help from a very active community of Wikidot users.\n++ Visit the MyWikidot.local Project Site\nGo to **[*http://my-wd-local.wikidot.com/ my-wd-local.wikidot.com]** -- for tips, discussions and how-to articles.\n---------------\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org] and the developers discussion at [*http://groups.google.com/group/wikidot Wikidot dev-list].\n+ Search all wikis\n[[module SearchAll]]\n+ Search users\n[[module SearchUsers]]
243	[[>]]\n[[module Rate]]\n[[/>]]\n[[code]]\n<body>\n<style type="text/css">\nbody\n{\n   font-family:verdana,arial,helvetica,sans-serif;\n   font-size:12.8px;\n   color:#333;\n   line-height:141%\n}\nblockquote\n{\n  border:1px dashed #999;\n  padding: 0 12.8px;\n  background-color:#f4f4f4\n}\na {\n    color: #b01;\n    text-decoration: none;\n    background: transparent;\n}\n.content-panel {\n    border: solid 1px #888880;\n    border-radius: 10px;\n    background-color: #999990;\n    margin: 10px 0 15px;\n    box-shadow: 3px 3px 6px #bbb;\n    box-shadow: 0 2px 6px rgba(0,0,0,0.5), inset 0 1px rgba(255,255,255,0.3), inset 0 10px rgba(255,255,255,0.2), inset 0 10px 20px rgba(255,255,255,0.25), inset 0 -15px 30px rgba(0,0,0,0.1);\n}\n.content-panel.standalone {\n    background: #fcfdfb;\n}\n.content-panel.series {\n    padding: 0 20px;\n    margin-bottom: 20px;\n}\na.newpage {\n    color: #d61;\n    text-decoration: none;\n    background: transparent;\n}\na:hover {\n    text-decoration: underline;\n    background-color: transparent;\n}\n.collapsed {\n  display: none;\n}\n.vanished {\n  display: none;\n}\n.warning-cont {\n  color: red;\n  text-align: center;\n}\n#warning-header {\n  font-size: 25.6px;\n}\n#warning {\n  font-size: 20px;\n}\n.retrofont {\nfont-family: 'Geo', cursive;\nfont-weight: bold;\n}\n.cblwarning {\n  color: red;\n}\n.green {\n  color: green;\n}\n.smaller-g {\nwidth: 45%;\nmargin: auto;\nbackground-color: #e6e6e6;\nborder: none;\nheight: 2px;\n}\n.adden{\nfont-size: 1.3em;\n}\n/* Standard Image Block */\n.scp-image-block {\n    border: solid 1px #666;\n    box-shadow: 0 1px 6px rgba(0,0,0,.25);\n    width: 300px;\n}\n \n.scp-image-block.block-right {\n        float: right;\n    clear: right;\n    margin: 0 2em 1em 2em;\n}\n \n.scp-image-block.block-left {\n    float: left;\n    clear: left;\n    margin: 0 2em 1em 0;\n}\n \n.scp-image-block.block-center {\n    margin-right: auto;\n    margin-left: auto;\n}\n.scp-image-block img {\n    border: 0;\n    width: 300px;\n}\n.scp-image-block .scp-image-caption {\n    background-color: #eee;\n    border-top: solid 1px #666;\n    padding: 2px 0;\n    font-size: 80%;\n    font-weight: bold;\n    text-align: center;\n    width: 300px;\n}\n.scp-image-block > p {\n    margin: 0;\n}\n.scp-image-block .scp-image-caption > p {\n    margin: 0;\n    padding: 0 10px;\n    line-height: 125%\n}\n</style>\n<script type="text/javascript">\nfunction addEvent(element, eventName, callback) {\n    if (element.addEventListener) {\n        element.addEventListener(eventName, callback, false);\n    } else if (element.attachEvent) {\n        element.attachEvent("on" + eventName, callback);\n    } else {\n        element["on" + eventName] = callback;\n    }\n}\n\nwindow.toggle = function(cls) {\n  var collapsed = document.querySelectorAll("." + cls + ".collapsed");\n  var expanded = document.querySelectorAll("." + cls + ":not(.collapsed)");\n  var i;\n  for(i = 0; i < collapsed.length; i++) {\n    collapsed[i].classList.remove("collapsed");\n  }\n  for(i = 0; i < expanded.length; i++) {\n    expanded[i].classList.add("collapsed");\n  }\n};\n\n// typewriter code\nfunction Typewriter(el, str, delay) {\n  if (!(this instanceof Typewriter)) return new Typewriter(el, str, delay);\n\n  this.el = el;\n  this.str = str || el.innerHTML || el.value;\n  this.delay = delay || 100;\n  this.i = 0;\n}\n\nTypewriter.prototype.type = function() {\n  var i = this.i,\n    char = this.str.charAt(i);\n\n  if (!char) {\n    clearInterval(this.intervalID);\n    return this;\n  }\n\n  if (char === '<') this.isTag = true;\n  if (char === '>') this.isTag = false;\n\n  this.el.innerHTML += char;\n  return this.i++;\n};\n\nTypewriter.prototype.start = function() {\n  var self = this;\n  if (this.i < 0) this.i = 0;\n  if (this.el.innerHTML === this.str) this.clear();\n\n  // this.emit('start');\n  (function loop() {\n    self.type();\n    if (self.isTag) return loop();\n    self.intervalID = setTimeout(loop, self.delay);\n  }());\n\n  return this;\n};\n\nTypewriter.prototype.stop = function() {\n  this.i = -1;\n  return this;\n};\n\nTypewriter.prototype.restart = function() {\n  this.clear();\n  this.i = 0;\n  return this.start();\n};\n\nTypewriter.prototype.clear = function() {\n  this.el.innerHTML = '';\n  return this;\n};\n\nvar junkChars = ['ó','ò','ñ','ç','¿','ß','à','è','ì','ù','Œ','þ','█','%','+','=','.','«','»','£','$','€','½','¾','À','È','Ì','Ò','Ù','Â','Ê','â','ê','Ä','Ë','ä','ë','ÿ','ü','•'];\n\nvar an_iteration = "";\n\n\nvar randomIntFromInterval = function(min,max)\n{\n  return Math.floor(Math.random()*(max-min+1)+min);\n};\n\nvar getJunkChars = function(length) {\n  var res = '';\n  var i;\n  for (i = 0; i < length; i++) {\n    res += junkChars[randomIntFromInterval(0,junkChars.length - 1)];\n  }\n  return res;\n};\n\nvar selTabs = [];\n\nfunction rndiam() {\n  var rndiams = document.getElementsByClassName('rndiam');\n  var i;\n  for (i = 0; i < rndiams.length; i++) {\n    rndiams[i].innerHTML = (Math.random() * (1.56 - 0.51) + 0.51).toFixed(2);\n  }\n  setTimeout(rndiam, 2000);\n}\n\nwindow.onload = function() {\n\nselTabs = [\n  document.getElementsByClassName('selTab1')[0],\n  document.getElementsByClassName('selTab2')[0],\n  document.getElementsByClassName('selTab3')[0],\n  document.getElementsByClassName('selTab4')[0],\n  document.getElementsByClassName('selTab5')[0]\n];\n\nvar msg1 = '= SCP-001 | Technical Clearance Required =';\n\nvar delay1 = 50;\nvar delay2 = 25;\n\nvar toD1 = delay1 * (msg1.length);\n\nvar stopEarly = function(){\n  document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished");\n};\n\nvar typ1 =Typewriter(document.querySelectorAll('#warning-header')[0],msg1,delay1)\n\ntyp1.start();\nsetTimeout(function() { document.getElementById("warning").classList.remove("vanished"); }, toD1 + 500);\nsetTimeout(function () { document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished"); }, toD1 + 1500);\n\n  rndiam();\n\n  var today = new Date();\n  var todayformat = "" + (today.getFullYear() - 1) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today").innerHTML = todayformat;\n  var todayformat2 = "" + (today.getFullYear() - 0) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today2").innerHTML = todayformat2;\n  document.getElementById("yearplus3").innerHTML = "" + (today.getFullYear() + 3);\n\n  an_iteration = document.getElementById("part1").innerHTML;\n};\n\nwindow.new_iteration = function() {\n  var new_iters = document.getElementsByClassName("newiter");\n  console.log(new_iters);\n  new_iters[new_iters.length - 1].classList.add("vanished");\n  var inner = document.getElementsByClassName("inner");\n  inner[inner.length - 1].innerHTML = "<hr /><hr />" + an_iteration;\n};\n</script>\n</script>\n<div class="warning-cont">\n<h2 id="warning-header">\n<p>\n</p>\n<h3 id="warning" class="vanished">  \n<p>This document exists as technical containment for an anomaly and thus does not adhere to standard SCiPNET formatting. Input 5/TECHNICAL clearance to proceed.\n</p>\n</div>\n\n<div id="collapsible_to_open_skip" class="article vanished">\n<p><center><a onclick="toggle('article');">[ INPUT PERSONAL IDENTIFICATION NUMBER ]</a></center></p>\n</div>\n<div class="article collapsed">\n <p><center><a onclick="toggle('article');">[ PERSONAL IDENTIFICATION NUMBER VERIFIED ]</a></center></p>\n<div id="part1">\n<hr />\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://topia.wikidot.com/local--files/jamcon-001/sky.jpg" style="width:300px;" alt="destruction.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>The sky above Point Alpha during a breach of SCP-001.</p>\n</div>\n</div>\n<p><b>Item #:</b> SCP-001</p>\n<p><b>Object Class:</b> Ontokinetic</p>\n<p><b>Containment Class:</b> Keter</p>\n<p><b>Special Containment Procedures:</b> If the sky begins changing color with no prior stimuli or indication, all Foundation sites are to go onto ALERT LEVEL 7 immediately. Personnel at Site-05 and -06 are to enter ALERT LEVEL 8 and begin evaluating new methods of containing SCP-001.</p>\n<p>Site-05 has been constructed around Point Alpha. Procedure 001-ENTRY is in effect to contain SCP-001-1. A monitor containing Document 032, as well as a speaker reading the document, are to be placed at least five meters away from SCP-001-1. If SCP-001-1 swells up to 2 meters or more in diameter, the speed of the narration is to increase. This system is to have a direct cable link to the Foundation SCiPNET database at Site-06, and several redundant power generators are to ensure this system stays operational at all times.</p>\n<p><b>Description:</b> SCP-001 is an ongoing ZK-Class "Reality Failure" Scenario. SCP-001 is prevented from happening through several procedures manipulating its sub-anomalies. The only known observable symptom of SCP-001 is the sky becoming a different color; it is believed that, if SCP-001 were to progress to the point where other symptoms began to occur, it would become irreversible.</p>\n<p>SCP-001-1 is a levitating object contained within Point Alpha, a cave chamber five kilometers south of the ruins of Babylon. SCP-001-1 visually resembles a perfectly smooth sphere with a texture similar to blurred television static. Measurement instruments indicate SCP-001-1 is currently <span class="rndiam"></span> meters in diameter. Solids and liquids that pass within the bounds of SCP-001-1 are replaced entirely with argon gas, giving the illusion of disappearing.</p>\n<p>SCP-001-2 is an entity that is capable of being seen within Point Alpha. SCP-001-2's manifestations are random; witnesses report seeing SCP-001-2 "past the walls of the cave." From descriptions of eyewitnesses, SCP-001-2 is humanoid in shape and large in stature, and is only visible when looking through SCP-001-1. Descriptions of SCP-001-2 vary from person to person; common features include an emaciated figure, long limbs, and prominent facial features. SCP-001-2 takes the position of lying against the wall. It is most often seen with a neutral expression; however, the expression occasionally changes to discomfort or bliss.</p>\n<p>It has been determined from historical precedent that "entertaining" SCP-001-2 through SCP-001-1 will prevent SCP-001.</p>\n<div class="scp-image-block block-left" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/cave.jpg" style="width:300px;" alt="cave.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Point Alpha shortly after it was acquired by the Foundation, with Agents Samuel and Boise preforming preliminary evaluation. SCP-001-1 not visible.</p>\n</div>\n</div>\n<p><b>History:</b> Point Alpha and its contents are believed to date back to prehistory. Babylonian scripts indicate that SCP-001-1 was routinely patrolled by a group of sages, known as the "Order of White and Black," who refused passage to anyone who could demonstrate the ability to read or write.</p>\n<p>This group was made defunct by the rise of Achaemenid Persia, which contained SCP-001-1 with a group of Zoroastrian monks. Very few accounts of SCP-001-1 are present; recovered accounts assert that all information about SCP-001-1 and its significance was passed orally.</p>\n<p>The most substantial account of SCP-001-1 and SCP-001-2 at this time dates back to the Greek philosopher Xera, who made an expedition into the Achaemenid Empire and found Point Alpha. During Alexander of Macedon's conquest of the Achaemenid Empire, he took interest in Xera's texts and continued the Achaemenid containment of SCP-001-1. However, this new institution, known as the "Cronus Guard", was given Greek epics to read in order to "punish Cronus" for his actions. This institution lasted through the partition of Macedon into the Seleucid Empire, the rise of the Sassanid Empire, and even the formation of the Rashidun Caliphate.</p>\n<p>The Cronus Guard were eventually replaced with an Islamic equivalent: "The Society for the Containment of the Babel Demon." This group was the first to preform substantial research on SCP-001-1, and the first to accurately link SCP-001 to SCP-001-1.</p>\n<div class="socotbd">\n<p style="font-size: 82%"><a onclick="toggle('socotbd');">View Attachment: Excerpts from the notes of the Society for the Containment of the Babel Demon</a></p>\n</div>\n<div class="socotbd collapsed">\n <p style="font-size: 82%"><a onclick="toggle('socotbd');">\nClose Attachment</a></p>\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/book.jpg" style="width:300px;" alt="book.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Records from the Society for the Containment of the Babel Demon, recovered in the early 19<sup>th</sup> century within ORIA archives.</p>\n</div>\n</div>\n<blockquote>\n<p><b>Literature Read:</b> The first quatrain of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON squirms but otherwise does not react. Consistent with reading of Homer's Epics by the Chronus Guard.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> The remaining quatrains of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> No changes from observed behavior.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> A poem written by Guardian Muhammad ibn Buya'aa meant explicitly for DEMON</p>\n<p><b>Result:</b> DEMON thrashes more than usual until the cessation of the reading. ORB begins to expand until the <i>Genealogies of the Nobles</i> is read to DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> None, for 10 days</p>\n<p><b>Result:</b> ORB is observed to expand, and DEMON is observed to thrash. Externally, the sky above the camp is noted to turn a darker red color, until the <i>Rubaiyat of Omar Khayyam</i> is read again.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON thrashes more than during previous readings of the <i>Rubaiyat of Omar Khayyam</i>. It is believed that the Quran may no longer have any effect on DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>One Thousand and One Nights</i></p>\n<p><b>Result:</b> DEMON ceases thrashing. New literature may be necessary to prevent further phenomena from occurring.</p>\n</blockquote>\n</div>\n<p>The Society was eventually absorbed into the Office for the Reclamation of Islamic Artifacts, who assumed containment of SCP-001-1. After Incident 001-EXAL, the Foundation purchased Point Alpha from ORIA for a large sum of currency and several Safe-class anomalies.</p>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Incident 001-EXAL</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<p style="font-size: 82%"><i>Note: Some documentation taken from ORIA's account of Incident 001-EXAL.</i></p>\n<p>On <span id="today"></span>, personnel with ORIA read the book <i>Tafsir al-Ahlam al-kabir</i>, or "Great Book of Interpretation of Dreams" as per normal containment of SCP-001-1. However, SCP-001-2 was observed to rapidly thrash while screaming. SCP-001-1 itself swelled from 50.3 centimeters to 3.2 meters in diameter.</p>\n<p>Shortly following this event, SCP-001 began to worsen. The sky worldwide turned a black/white color similar to the texture of SCP-001-1. In addition, reality bending phenomena began worldwide, causing deformed geography, the manifestation of dangerous anomalous objects and several natural disasters. This event was ended after ORIA personnel read SCP-001-1 an undisclosed number of as-of-yet unread books, which caused SCP-001-2 to stop thrashing and SCP-001 to restore to its pre-incident point. However, the damage caused by this incident was deemed enough to require an activation of <a target="_top" href="http://www.scp-wiki.net/scp-2000">SCP-2000</a>.</p>\n<p>This prompted the Foundation to take control of containment of SCP-001. See <b>History</b> segment above for more information.</p>\n<div class="info">\n<p style="font-size: 82%"><a onclick="toggle('info');">Input Level 5/001 Credentials</a></p>\n</div>\n<div class="info collapsed">\n <p style="font-size: 82%"><a onclick="toggle('info');">\nAccess Granted</a></p>\n<p>During Incident 001-Alpha, vocalizations in <a target="_top" href="http://www.scp-wiki.net/scp-140">Daevish</a> were recorded by ORIA instruments within Point Alpha. The source is presently unknown. The following are approximate translations of these vocalizations.</p>\n<blockquote>\n<p>It has passed some [space/time].</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. It [has had/used to be] fun. It is [time/space] to leave.</p>\n</blockquote>\n<blockquote>\n<p>You cannot [stay/root/hold] yourself forever.</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. You [must/will] awaken.</p>\n</blockquote>\n<blockquote>\n<p>No, you have [entered/reentered] slumber for too long. Wake up.</p>\n</blockquote>\n<blockquote>\n<p>[King/Prince/loved one], it is time for you to wake up.</p>\n</blockquote>\n<blockquote>\n<p>It is fun to be in the [UNKNOWN: gaera] but you cannot be [in/rooted] there [forever/unending]. It is [entertaining/deathlike], but you must wake up.</p>\n</blockquote>\n<blockquote>\n<p>Wake up, [King/Prince/loved one]. We miss you.</p>\n</blockquote>\n</div>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Addenda</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<div class="discus">\n<p style="font-size: 82%"><a onclick="toggle('discus');">Input Level 5/001 Clearance</a></p>\n</div>\n<div class="discus collapsed">\n <p style="font-size: 82%"><a onclick="toggle('discus');">\nAccess Granted</a></p>\n<div div class="content-panel standalone series">\n<center>\n<h2>Discussion Thread 001-398:<br />Continued Containment</h2>\n<p style="font-size: 82%">Started on: <span id="today2"></span></p>\n<p style="font-size: 82%">Started by: <tt>HMCL Robinson</tt></p>\n</center>\n</div>\n<blockquote>\n<p><b><tt>HMCL Robinson:</tt></b> The usage of this thread will be for containment directives for SCP-001. In the past year that we have spent containing it, SCP-001-1's literature needs have become increasingly draconic: we can very rarely reread books without it beginning to thrash, and even then we're running out of high-quality literature to give it. We need a more long-term, reliable solution for containment. SCP-001's file has been temporarily opened up to all personnel with Level 2 clearance. Anybody can submit an idea.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Use Artificial Intelligence Constructs to automatically generate new stories for SCP-001-2.</p>\n<p><b>Status: <tt>[ <span class="green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> TSATPWTCOTTTADC.aic was able to generate 10,000 volumes of stories imitating Greek literature. However, when the first was read to SCP-001-1, it began thrashing beyond acceptable measures until it was read approved reading. No further computer-generated works are to be given to SCP-001-1.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Hire a full team of authors to create literature for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> In the past, literature written specifically for SCP-001-1 have failed to contain it (see: SCtBD Document #249). With SCP-001-2's current state this is not to be attempted again.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> This could represent a possible information leak. Test denied.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Reason:</b> With SCP-001-2's recent containment breach, these measures are approved.</p>\n<p><b>Result:</b> SCP-001-2 observed to stop thrashing and stay completely still, seemingly smiling. In addition, SCP-███ was able to be read for 14 consecutive readings before SCP-001-2 resumed normal activity.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Regularly read mainlist documentation for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Replacement of the SCP-001 file with several "001 Proposals" with much more grand implications than normal SCP files.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> By only occasionally reading 001 files, SCP-001-2 is observed to be more calm on average. The previously observed actions of retaliation in response to specially crafted literature do not apply here.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Given the variation of the current 2957 SCP mainlist files, SCP-001-1 is to be read a random arrangement of these files, in repeat.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> This strategy succeeded for approximately 3 years, allowing for several repeats. However, on <span id="yearplus3"></span>-9-30, SCP-001-2 began thrashing rapidly, requiring newer documentation to sedate.</p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-4999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> SCP-001-1 has required progressively more SCP mainlist articles for containment, even to the point of the fabrication of some anomalous entities.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Rewriting of several older SCP mainlist files to be more exaggerated and narrative-like.</p>\n<p><b>Status: <tt>[ <span style="color: orange">TENTATIVELY APPROVED</span> ]</tt></b></p>\n<p><b>Note:</b> <i>Deliberately exaggerating our files for the purpose of this entity is crossing some lines, but for now, it will have to do. However, we need a better way. I'm calling a committee. We need to put a stop to this.</i></p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Creation of a file describing SCP-001 that uses recursion to effectively be "never-ending." Psychoanalysis related to SCP-001-2's apparent "enjoyment" of works implies a slight ego and bias towards works about itself, and past strategies with this idea have worked for long periods of time. File is to be archived as a "technical" file.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> Procedure 001-ENTRY implemented. [DATA EXPUNGED]</p>\n</blockquote>\n</div>\n\n<div class="newiter">\n<p style="font-size: 82%"><a onclick="new_iteration();">Input Level 5/TECHNICAL Clearance</a></p>\n</div>\n<div class="inner">\n<!-- :-) -->\n</div>\n</div>\n</body>\n[[/code]]
244	[[>]]\n[[module Rate]]\n[[/>]]\n[[html]]\n<body>\n<style type="text/css">\nbody\n{\n   font-family:verdana,arial,helvetica,sans-serif;\n   font-size:12.8px;\n   color:#333;\n   line-height:141%\n}\nblockquote\n{\n  border:1px dashed #999;\n  padding: 0 12.8px;\n  background-color:#f4f4f4\n}\na {\n    color: #b01;\n    text-decoration: none;\n    background: transparent;\n}\n.content-panel {\n    border: solid 1px #888880;\n    border-radius: 10px;\n    background-color: #999990;\n    margin: 10px 0 15px;\n    box-shadow: 3px 3px 6px #bbb;\n    box-shadow: 0 2px 6px rgba(0,0,0,0.5), inset 0 1px rgba(255,255,255,0.3), inset 0 10px rgba(255,255,255,0.2), inset 0 10px 20px rgba(255,255,255,0.25), inset 0 -15px 30px rgba(0,0,0,0.1);\n}\n.content-panel.standalone {\n    background: #fcfdfb;\n}\n.content-panel.series {\n    padding: 0 20px;\n    margin-bottom: 20px;\n}\na.newpage {\n    color: #d61;\n    text-decoration: none;\n    background: transparent;\n}\na:hover {\n    text-decoration: underline;\n    background-color: transparent;\n}\n.collapsed {\n  display: none;\n}\n.vanished {\n  display: none;\n}\n.warning-cont {\n  color: red;\n  text-align: center;\n}\n#warning-header {\n  font-size: 25.6px;\n}\n#warning {\n  font-size: 20px;\n}\n.retrofont {\nfont-family: 'Geo', cursive;\nfont-weight: bold;\n}\n.cblwarning {\n  color: red;\n}\n.green {\n  color: green;\n}\n.smaller-g {\nwidth: 45%;\nmargin: auto;\nbackground-color: #e6e6e6;\nborder: none;\nheight: 2px;\n}\n.adden{\nfont-size: 1.3em;\n}\n/* Standard Image Block */\n.scp-image-block {\n    border: solid 1px #666;\n    box-shadow: 0 1px 6px rgba(0,0,0,.25);\n    width: 300px;\n}\n \n.scp-image-block.block-right {\n        float: right;\n    clear: right;\n    margin: 0 2em 1em 2em;\n}\n \n.scp-image-block.block-left {\n    float: left;\n    clear: left;\n    margin: 0 2em 1em 0;\n}\n \n.scp-image-block.block-center {\n    margin-right: auto;\n    margin-left: auto;\n}\n.scp-image-block img {\n    border: 0;\n    width: 300px;\n}\n.scp-image-block .scp-image-caption {\n    background-color: #eee;\n    border-top: solid 1px #666;\n    padding: 2px 0;\n    font-size: 80%;\n    font-weight: bold;\n    text-align: center;\n    width: 300px;\n}\n.scp-image-block > p {\n    margin: 0;\n}\n.scp-image-block .scp-image-caption > p {\n    margin: 0;\n    padding: 0 10px;\n    line-height: 125%\n}\n</style>\n<script type="text/javascript">\nfunction addEvent(element, eventName, callback) {\n    if (element.addEventListener) {\n        element.addEventListener(eventName, callback, false);\n    } else if (element.attachEvent) {\n        element.attachEvent("on" + eventName, callback);\n    } else {\n        element["on" + eventName] = callback;\n    }\n}\n\nwindow.toggle = function(cls) {\n  var collapsed = document.querySelectorAll("." + cls + ".collapsed");\n  var expanded = document.querySelectorAll("." + cls + ":not(.collapsed)");\n  var i;\n  for(i = 0; i < collapsed.length; i++) {\n    collapsed[i].classList.remove("collapsed");\n  }\n  for(i = 0; i < expanded.length; i++) {\n    expanded[i].classList.add("collapsed");\n  }\n};\n\n// typewriter code\nfunction Typewriter(el, str, delay) {\n  if (!(this instanceof Typewriter)) return new Typewriter(el, str, delay);\n\n  this.el = el;\n  this.str = str || el.innerHTML || el.value;\n  this.delay = delay || 100;\n  this.i = 0;\n}\n\nTypewriter.prototype.type = function() {\n  var i = this.i,\n    char = this.str.charAt(i);\n\n  if (!char) {\n    clearInterval(this.intervalID);\n    return this;\n  }\n\n  if (char === '<') this.isTag = true;\n  if (char === '>') this.isTag = false;\n\n  this.el.innerHTML += char;\n  return this.i++;\n};\n\nTypewriter.prototype.start = function() {\n  var self = this;\n  if (this.i < 0) this.i = 0;\n  if (this.el.innerHTML === this.str) this.clear();\n\n  // this.emit('start');\n  (function loop() {\n    self.type();\n    if (self.isTag) return loop();\n    self.intervalID = setTimeout(loop, self.delay);\n  }());\n\n  return this;\n};\n\nTypewriter.prototype.stop = function() {\n  this.i = -1;\n  return this;\n};\n\nTypewriter.prototype.restart = function() {\n  this.clear();\n  this.i = 0;\n  return this.start();\n};\n\nTypewriter.prototype.clear = function() {\n  this.el.innerHTML = '';\n  return this;\n};\n\nvar junkChars = ['ó','ò','ñ','ç','¿','ß','à','è','ì','ù','Œ','þ','█','%','+','=','.','«','»','£','$','€','½','¾','À','È','Ì','Ò','Ù','Â','Ê','â','ê','Ä','Ë','ä','ë','ÿ','ü','•'];\n\nvar an_iteration = "";\n\n\nvar randomIntFromInterval = function(min,max)\n{\n  return Math.floor(Math.random()*(max-min+1)+min);\n};\n\nvar getJunkChars = function(length) {\n  var res = '';\n  var i;\n  for (i = 0; i < length; i++) {\n    res += junkChars[randomIntFromInterval(0,junkChars.length - 1)];\n  }\n  return res;\n};\n\nvar selTabs = [];\n\nfunction rndiam() {\n  var rndiams = document.getElementsByClassName('rndiam');\n  var i;\n  for (i = 0; i < rndiams.length; i++) {\n    rndiams[i].innerHTML = (Math.random() * (1.56 - 0.51) + 0.51).toFixed(2);\n  }\n  setTimeout(rndiam, 2000);\n}\n\nwindow.onload = function() {\n\nselTabs = [\n  document.getElementsByClassName('selTab1')[0],\n  document.getElementsByClassName('selTab2')[0],\n  document.getElementsByClassName('selTab3')[0],\n  document.getElementsByClassName('selTab4')[0],\n  document.getElementsByClassName('selTab5')[0]\n];\n\nvar msg1 = '= SCP-001 | Technical Clearance Required =';\n\nvar delay1 = 50;\nvar delay2 = 25;\n\nvar toD1 = delay1 * (msg1.length);\n\nvar stopEarly = function(){\n  document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished");\n};\n\nvar typ1 =Typewriter(document.querySelectorAll('#warning-header')[0],msg1,delay1)\n\ntyp1.start();\nsetTimeout(function() { document.getElementById("warning").classList.remove("vanished"); }, toD1 + 500);\nsetTimeout(function () { document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished"); }, toD1 + 1500);\n\n  rndiam();\n\n  var today = new Date();\n  var todayformat = "" + (today.getFullYear() - 1) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today").innerHTML = todayformat;\n  var todayformat2 = "" + (today.getFullYear() - 0) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today2").innerHTML = todayformat2;\n  document.getElementById("yearplus3").innerHTML = "" + (today.getFullYear() + 3);\n\n  an_iteration = document.getElementById("part1").innerHTML;\n};\n\nwindow.new_iteration = function() {\n  var new_iters = document.getElementsByClassName("newiter");\n  console.log(new_iters);\n  new_iters[new_iters.length - 1].classList.add("vanished");\n  var inner = document.getElementsByClassName("inner");\n  inner[inner.length - 1].innerHTML = "<hr /><hr />" + an_iteration;\n};\n</script>\n</script>\n<div class="warning-cont">\n<h2 id="warning-header">\n<p>\n</p>\n<h3 id="warning" class="vanished">  \n<p>This document exists as technical containment for an anomaly and thus does not adhere to standard SCiPNET formatting. Input 5/TECHNICAL clearance to proceed.\n</p>\n</div>\n\n<div id="collapsible_to_open_skip" class="article vanished">\n<p><center><a onclick="toggle('article');">[ INPUT PERSONAL IDENTIFICATION NUMBER ]</a></center></p>\n</div>\n<div class="article collapsed">\n <p><center><a onclick="toggle('article');">[ PERSONAL IDENTIFICATION NUMBER VERIFIED ]</a></center></p>\n<div id="part1">\n<hr />\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://topia.wikidot.com/local--files/jamcon-001/sky.jpg" style="width:300px;" alt="destruction.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>The sky above Point Alpha during a breach of SCP-001.</p>\n</div>\n</div>\n<p><b>Item #:</b> SCP-001</p>\n<p><b>Object Class:</b> Ontokinetic</p>\n<p><b>Containment Class:</b> Keter</p>\n<p><b>Special Containment Procedures:</b> If the sky begins changing color with no prior stimuli or indication, all Foundation sites are to go onto ALERT LEVEL 7 immediately. Personnel at Site-05 and -06 are to enter ALERT LEVEL 8 and begin evaluating new methods of containing SCP-001.</p>\n<p>Site-05 has been constructed around Point Alpha. Procedure 001-ENTRY is in effect to contain SCP-001-1. A monitor containing Document 032, as well as a speaker reading the document, are to be placed at least five meters away from SCP-001-1. If SCP-001-1 swells up to 2 meters or more in diameter, the speed of the narration is to increase. This system is to have a direct cable link to the Foundation SCiPNET database at Site-06, and several redundant power generators are to ensure this system stays operational at all times.</p>\n<p><b>Description:</b> SCP-001 is an ongoing ZK-Class "Reality Failure" Scenario. SCP-001 is prevented from happening through several procedures manipulating its sub-anomalies. The only known observable symptom of SCP-001 is the sky becoming a different color; it is believed that, if SCP-001 were to progress to the point where other symptoms began to occur, it would become irreversible.</p>\n<p>SCP-001-1 is a levitating object contained within Point Alpha, a cave chamber five kilometers south of the ruins of Babylon. SCP-001-1 visually resembles a perfectly smooth sphere with a texture similar to blurred television static. Measurement instruments indicate SCP-001-1 is currently <span class="rndiam"></span> meters in diameter. Solids and liquids that pass within the bounds of SCP-001-1 are replaced entirely with argon gas, giving the illusion of disappearing.</p>\n<p>SCP-001-2 is an entity that is capable of being seen within Point Alpha. SCP-001-2's manifestations are random; witnesses report seeing SCP-001-2 "past the walls of the cave." From descriptions of eyewitnesses, SCP-001-2 is humanoid in shape and large in stature, and is only visible when looking through SCP-001-1. Descriptions of SCP-001-2 vary from person to person; common features include an emaciated figure, long limbs, and prominent facial features. SCP-001-2 takes the position of lying against the wall. It is most often seen with a neutral expression; however, the expression occasionally changes to discomfort or bliss.</p>\n<p>It has been determined from historical precedent that "entertaining" SCP-001-2 through SCP-001-1 will prevent SCP-001.</p>\n<div class="scp-image-block block-left" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/cave.jpg" style="width:300px;" alt="cave.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Point Alpha shortly after it was acquired by the Foundation, with Agents Samuel and Boise preforming preliminary evaluation. SCP-001-1 not visible.</p>\n</div>\n</div>\n<p><b>History:</b> Point Alpha and its contents are believed to date back to prehistory. Babylonian scripts indicate that SCP-001-1 was routinely patrolled by a group of sages, known as the "Order of White and Black," who refused passage to anyone who could demonstrate the ability to read or write.</p>\n<p>This group was made defunct by the rise of Achaemenid Persia, which contained SCP-001-1 with a group of Zoroastrian monks. Very few accounts of SCP-001-1 are present; recovered accounts assert that all information about SCP-001-1 and its significance was passed orally.</p>\n<p>The most substantial account of SCP-001-1 and SCP-001-2 at this time dates back to the Greek philosopher Xera, who made an expedition into the Achaemenid Empire and found Point Alpha. During Alexander of Macedon's conquest of the Achaemenid Empire, he took interest in Xera's texts and continued the Achaemenid containment of SCP-001-1. However, this new institution, known as the "Cronus Guard", was given Greek epics to read in order to "punish Cronus" for his actions. This institution lasted through the partition of Macedon into the Seleucid Empire, the rise of the Sassanid Empire, and even the formation of the Rashidun Caliphate.</p>\n<p>The Cronus Guard were eventually replaced with an Islamic equivalent: "The Society for the Containment of the Babel Demon." This group was the first to preform substantial research on SCP-001-1, and the first to accurately link SCP-001 to SCP-001-1.</p>\n<div class="socotbd">\n<p style="font-size: 82%"><a onclick="toggle('socotbd');">View Attachment: Excerpts from the notes of the Society for the Containment of the Babel Demon</a></p>\n</div>\n<div class="socotbd collapsed">\n <p style="font-size: 82%"><a onclick="toggle('socotbd');">\nClose Attachment</a></p>\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/book.jpg" style="width:300px;" alt="book.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Records from the Society for the Containment of the Babel Demon, recovered in the early 19<sup>th</sup> century within ORIA archives.</p>\n</div>\n</div>\n<blockquote>\n<p><b>Literature Read:</b> The first quatrain of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON squirms but otherwise does not react. Consistent with reading of Homer's Epics by the Chronus Guard.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> The remaining quatrains of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> No changes from observed behavior.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> A poem written by Guardian Muhammad ibn Buya'aa meant explicitly for DEMON</p>\n<p><b>Result:</b> DEMON thrashes more than usual until the cessation of the reading. ORB begins to expand until the <i>Genealogies of the Nobles</i> is read to DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> None, for 10 days</p>\n<p><b>Result:</b> ORB is observed to expand, and DEMON is observed to thrash. Externally, the sky above the camp is noted to turn a darker red color, until the <i>Rubaiyat of Omar Khayyam</i> is read again.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON thrashes more than during previous readings of the <i>Rubaiyat of Omar Khayyam</i>. It is believed that the Quran may no longer have any effect on DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>One Thousand and One Nights</i></p>\n<p><b>Result:</b> DEMON ceases thrashing. New literature may be necessary to prevent further phenomena from occurring.</p>\n</blockquote>\n</div>\n<p>The Society was eventually absorbed into the Office for the Reclamation of Islamic Artifacts, who assumed containment of SCP-001-1. After Incident 001-EXAL, the Foundation purchased Point Alpha from ORIA for a large sum of currency and several Safe-class anomalies.</p>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Incident 001-EXAL</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<p style="font-size: 82%"><i>Note: Some documentation taken from ORIA's account of Incident 001-EXAL.</i></p>\n<p>On <span id="today"></span>, personnel with ORIA read the book <i>Tafsir al-Ahlam al-kabir</i>, or "Great Book of Interpretation of Dreams" as per normal containment of SCP-001-1. However, SCP-001-2 was observed to rapidly thrash while screaming. SCP-001-1 itself swelled from 50.3 centimeters to 3.2 meters in diameter.</p>\n<p>Shortly following this event, SCP-001 began to worsen. The sky worldwide turned a black/white color similar to the texture of SCP-001-1. In addition, reality bending phenomena began worldwide, causing deformed geography, the manifestation of dangerous anomalous objects and several natural disasters. This event was ended after ORIA personnel read SCP-001-1 an undisclosed number of as-of-yet unread books, which caused SCP-001-2 to stop thrashing and SCP-001 to restore to its pre-incident point. However, the damage caused by this incident was deemed enough to require an activation of <a target="_top" href="http://www.scp-wiki.net/scp-2000">SCP-2000</a>.</p>\n<p>This prompted the Foundation to take control of containment of SCP-001. See <b>History</b> segment above for more information.</p>\n<div class="info">\n<p style="font-size: 82%"><a onclick="toggle('info');">Input Level 5/001 Credentials</a></p>\n</div>\n<div class="info collapsed">\n <p style="font-size: 82%"><a onclick="toggle('info');">\nAccess Granted</a></p>\n<p>During Incident 001-Alpha, vocalizations in <a target="_top" href="http://www.scp-wiki.net/scp-140">Daevish</a> were recorded by ORIA instruments within Point Alpha. The source is presently unknown. The following are approximate translations of these vocalizations.</p>\n<blockquote>\n<p>It has passed some [space/time].</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. It [has had/used to be] fun. It is [time/space] to leave.</p>\n</blockquote>\n<blockquote>\n<p>You cannot [stay/root/hold] yourself forever.</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. You [must/will] awaken.</p>\n</blockquote>\n<blockquote>\n<p>No, you have [entered/reentered] slumber for too long. Wake up.</p>\n</blockquote>\n<blockquote>\n<p>[King/Prince/loved one], it is time for you to wake up.</p>\n</blockquote>\n<blockquote>\n<p>It is fun to be in the [UNKNOWN: gaera] but you cannot be [in/rooted] there [forever/unending]. It is [entertaining/deathlike], but you must wake up.</p>\n</blockquote>\n<blockquote>\n<p>Wake up, [King/Prince/loved one]. We miss you.</p>\n</blockquote>\n</div>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Addenda</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<div class="discus">\n<p style="font-size: 82%"><a onclick="toggle('discus');">Input Level 5/001 Clearance</a></p>\n</div>\n<div class="discus collapsed">\n <p style="font-size: 82%"><a onclick="toggle('discus');">\nAccess Granted</a></p>\n<div div class="content-panel standalone series">\n<center>\n<h2>Discussion Thread 001-398:<br />Continued Containment</h2>\n<p style="font-size: 82%">Started on: <span id="today2"></span></p>\n<p style="font-size: 82%">Started by: <tt>HMCL Robinson</tt></p>\n</center>\n</div>\n<blockquote>\n<p><b><tt>HMCL Robinson:</tt></b> The usage of this thread will be for containment directives for SCP-001. In the past year that we have spent containing it, SCP-001-1's literature needs have become increasingly draconic: we can very rarely reread books without it beginning to thrash, and even then we're running out of high-quality literature to give it. We need a more long-term, reliable solution for containment. SCP-001's file has been temporarily opened up to all personnel with Level 2 clearance. Anybody can submit an idea.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Use Artificial Intelligence Constructs to automatically generate new stories for SCP-001-2.</p>\n<p><b>Status: <tt>[ <span class="green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> TSATPWTCOTTTADC.aic was able to generate 10,000 volumes of stories imitating Greek literature. However, when the first was read to SCP-001-1, it began thrashing beyond acceptable measures until it was read approved reading. No further computer-generated works are to be given to SCP-001-1.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Hire a full team of authors to create literature for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> In the past, literature written specifically for SCP-001-1 have failed to contain it (see: SCtBD Document #249). With SCP-001-2's current state this is not to be attempted again.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> This could represent a possible information leak. Test denied.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Reason:</b> With SCP-001-2's recent containment breach, these measures are approved.</p>\n<p><b>Result:</b> SCP-001-2 observed to stop thrashing and stay completely still, seemingly smiling. In addition, SCP-███ was able to be read for 14 consecutive readings before SCP-001-2 resumed normal activity.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Regularly read mainlist documentation for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Replacement of the SCP-001 file with several "001 Proposals" with much more grand implications than normal SCP files.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> By only occasionally reading 001 files, SCP-001-2 is observed to be more calm on average. The previously observed actions of retaliation in response to specially crafted literature do not apply here.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Given the variation of the current 2957 SCP mainlist files, SCP-001-1 is to be read a random arrangement of these files, in repeat.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> This strategy succeeded for approximately 3 years, allowing for several repeats. However, on <span id="yearplus3"></span>-9-30, SCP-001-2 began thrashing rapidly, requiring newer documentation to sedate.</p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-4999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> SCP-001-1 has required progressively more SCP mainlist articles for containment, even to the point of the fabrication of some anomalous entities.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Rewriting of several older SCP mainlist files to be more exaggerated and narrative-like.</p>\n<p><b>Status: <tt>[ <span style="color: orange">TENTATIVELY APPROVED</span> ]</tt></b></p>\n<p><b>Note:</b> <i>Deliberately exaggerating our files for the purpose of this entity is crossing some lines, but for now, it will have to do. However, we need a better way. I'm calling a committee. We need to put a stop to this.</i></p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Creation of a file describing SCP-001 that uses recursion to effectively be "never-ending." Psychoanalysis related to SCP-001-2's apparent "enjoyment" of works implies a slight ego and bias towards works about itself, and past strategies with this idea have worked for long periods of time. File is to be archived as a "technical" file.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> Procedure 001-ENTRY implemented. [DATA EXPUNGED]</p>\n</blockquote>\n</div>\n\n<div class="newiter">\n<p style="font-size: 82%"><a onclick="new_iteration();">Input Level 5/TECHNICAL Clearance</a></p>\n</div>\n<div class="inner">\n<!-- :-) -->\n</div>\n</div>\n</body>\n[[/html]]
251	**Test,** new revision 25.\n\n[[html]]\n<body>Test html.</body>\n[[/html]]
252	**Test,** new revision 25.\n\n<html>\n<body>Test html.</body>\n</html>
253	[[>]]\n[[module Rate]]\n[[/>]]\n<html>\n<body>\n<style type="text/css">\nbody\n{\n   font-family:verdana,arial,helvetica,sans-serif;\n   font-size:12.8px;\n   color:#333;\n   line-height:141%\n}\nblockquote\n{\n  border:1px dashed #999;\n  padding: 0 12.8px;\n  background-color:#f4f4f4\n}\na {\n    color: #b01;\n    text-decoration: none;\n    background: transparent;\n}\n.content-panel {\n    border: solid 1px #888880;\n    border-radius: 10px;\n    background-color: #999990;\n    margin: 10px 0 15px;\n    box-shadow: 3px 3px 6px #bbb;\n    box-shadow: 0 2px 6px rgba(0,0,0,0.5), inset 0 1px rgba(255,255,255,0.3), inset 0 10px rgba(255,255,255,0.2), inset 0 10px 20px rgba(255,255,255,0.25), inset 0 -15px 30px rgba(0,0,0,0.1);\n}\n.content-panel.standalone {\n    background: #fcfdfb;\n}\n.content-panel.series {\n    padding: 0 20px;\n    margin-bottom: 20px;\n}\na.newpage {\n    color: #d61;\n    text-decoration: none;\n    background: transparent;\n}\na:hover {\n    text-decoration: underline;\n    background-color: transparent;\n}\n.collapsed {\n  display: none;\n}\n.vanished {\n  display: none;\n}\n.warning-cont {\n  color: red;\n  text-align: center;\n}\n#warning-header {\n  font-size: 25.6px;\n}\n#warning {\n  font-size: 20px;\n}\n.retrofont {\nfont-family: 'Geo', cursive;\nfont-weight: bold;\n}\n.cblwarning {\n  color: red;\n}\n.green {\n  color: green;\n}\n.smaller-g {\nwidth: 45%;\nmargin: auto;\nbackground-color: #e6e6e6;\nborder: none;\nheight: 2px;\n}\n.adden{\nfont-size: 1.3em;\n}\n/* Standard Image Block */\n.scp-image-block {\n    border: solid 1px #666;\n    box-shadow: 0 1px 6px rgba(0,0,0,.25);\n    width: 300px;\n}\n \n.scp-image-block.block-right {\n        float: right;\n    clear: right;\n    margin: 0 2em 1em 2em;\n}\n \n.scp-image-block.block-left {\n    float: left;\n    clear: left;\n    margin: 0 2em 1em 0;\n}\n \n.scp-image-block.block-center {\n    margin-right: auto;\n    margin-left: auto;\n}\n.scp-image-block img {\n    border: 0;\n    width: 300px;\n}\n.scp-image-block .scp-image-caption {\n    background-color: #eee;\n    border-top: solid 1px #666;\n    padding: 2px 0;\n    font-size: 80%;\n    font-weight: bold;\n    text-align: center;\n    width: 300px;\n}\n.scp-image-block > p {\n    margin: 0;\n}\n.scp-image-block .scp-image-caption > p {\n    margin: 0;\n    padding: 0 10px;\n    line-height: 125%\n}\n</style>\n<script type="text/javascript">\nfunction addEvent(element, eventName, callback) {\n    if (element.addEventListener) {\n        element.addEventListener(eventName, callback, false);\n    } else if (element.attachEvent) {\n        element.attachEvent("on" + eventName, callback);\n    } else {\n        element["on" + eventName] = callback;\n    }\n}\n\nwindow.toggle = function(cls) {\n  var collapsed = document.querySelectorAll("." + cls + ".collapsed");\n  var expanded = document.querySelectorAll("." + cls + ":not(.collapsed)");\n  var i;\n  for(i = 0; i < collapsed.length; i++) {\n    collapsed[i].classList.remove("collapsed");\n  }\n  for(i = 0; i < expanded.length; i++) {\n    expanded[i].classList.add("collapsed");\n  }\n};\n\n// typewriter code\nfunction Typewriter(el, str, delay) {\n  if (!(this instanceof Typewriter)) return new Typewriter(el, str, delay);\n\n  this.el = el;\n  this.str = str || el.innerHTML || el.value;\n  this.delay = delay || 100;\n  this.i = 0;\n}\n\nTypewriter.prototype.type = function() {\n  var i = this.i,\n    char = this.str.charAt(i);\n\n  if (!char) {\n    clearInterval(this.intervalID);\n    return this;\n  }\n\n  if (char === '<') this.isTag = true;\n  if (char === '>') this.isTag = false;\n\n  this.el.innerHTML += char;\n  return this.i++;\n};\n\nTypewriter.prototype.start = function() {\n  var self = this;\n  if (this.i < 0) this.i = 0;\n  if (this.el.innerHTML === this.str) this.clear();\n\n  // this.emit('start');\n  (function loop() {\n    self.type();\n    if (self.isTag) return loop();\n    self.intervalID = setTimeout(loop, self.delay);\n  }());\n\n  return this;\n};\n\nTypewriter.prototype.stop = function() {\n  this.i = -1;\n  return this;\n};\n\nTypewriter.prototype.restart = function() {\n  this.clear();\n  this.i = 0;\n  return this.start();\n};\n\nTypewriter.prototype.clear = function() {\n  this.el.innerHTML = '';\n  return this;\n};\n\nvar junkChars = ['ó','ò','ñ','ç','¿','ß','à','è','ì','ù','Œ','þ','█','%','+','=','.','«','»','£','$','€','½','¾','À','È','Ì','Ò','Ù','Â','Ê','â','ê','Ä','Ë','ä','ë','ÿ','ü','•'];\n\nvar an_iteration = "";\n\n\nvar randomIntFromInterval = function(min,max)\n{\n  return Math.floor(Math.random()*(max-min+1)+min);\n};\n\nvar getJunkChars = function(length) {\n  var res = '';\n  var i;\n  for (i = 0; i < length; i++) {\n    res += junkChars[randomIntFromInterval(0,junkChars.length - 1)];\n  }\n  return res;\n};\n\nvar selTabs = [];\n\nfunction rndiam() {\n  var rndiams = document.getElementsByClassName('rndiam');\n  var i;\n  for (i = 0; i < rndiams.length; i++) {\n    rndiams[i].innerHTML = (Math.random() * (1.56 - 0.51) + 0.51).toFixed(2);\n  }\n  setTimeout(rndiam, 2000);\n}\n\nwindow.onload = function() {\n\nselTabs = [\n  document.getElementsByClassName('selTab1')[0],\n  document.getElementsByClassName('selTab2')[0],\n  document.getElementsByClassName('selTab3')[0],\n  document.getElementsByClassName('selTab4')[0],\n  document.getElementsByClassName('selTab5')[0]\n];\n\nvar msg1 = '= SCP-001 | Technical Clearance Required =';\n\nvar delay1 = 50;\nvar delay2 = 25;\n\nvar toD1 = delay1 * (msg1.length);\n\nvar stopEarly = function(){\n  document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished");\n};\n\nvar typ1 =Typewriter(document.querySelectorAll('#warning-header')[0],msg1,delay1)\n\ntyp1.start();\nsetTimeout(function() { document.getElementById("warning").classList.remove("vanished"); }, toD1 + 500);\nsetTimeout(function () { document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished"); }, toD1 + 1500);\n\n  rndiam();\n\n  var today = new Date();\n  var todayformat = "" + (today.getFullYear() - 1) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today").innerHTML = todayformat;\n  var todayformat2 = "" + (today.getFullYear() - 0) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today2").innerHTML = todayformat2;\n  document.getElementById("yearplus3").innerHTML = "" + (today.getFullYear() + 3);\n\n  an_iteration = document.getElementById("part1").innerHTML;\n};\n\nwindow.new_iteration = function() {\n  var new_iters = document.getElementsByClassName("newiter");\n  console.log(new_iters);\n  new_iters[new_iters.length - 1].classList.add("vanished");\n  var inner = document.getElementsByClassName("inner");\n  inner[inner.length - 1].innerHTML = "<hr /><hr />" + an_iteration;\n};\n</script>\n</script>\n<div class="warning-cont">\n<h2 id="warning-header">\n<p>\n</p>\n<h3 id="warning" class="vanished">  \n<p>This document exists as technical containment for an anomaly and thus does not adhere to standard SCiPNET formatting. Input 5/TECHNICAL clearance to proceed.\n</p>\n</div>\n\n<div id="collapsible_to_open_skip" class="article vanished">\n<p><center><a onclick="toggle('article');">[ INPUT PERSONAL IDENTIFICATION NUMBER ]</a></center></p>\n</div>\n<div class="article collapsed">\n <p><center><a onclick="toggle('article');">[ PERSONAL IDENTIFICATION NUMBER VERIFIED ]</a></center></p>\n<div id="part1">\n<hr />\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://topia.wikidot.com/local--files/jamcon-001/sky.jpg" style="width:300px;" alt="destruction.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>The sky above Point Alpha during a breach of SCP-001.</p>\n</div>\n</div>\n<p><b>Item #:</b> SCP-001</p>\n<p><b>Object Class:</b> Ontokinetic</p>\n<p><b>Containment Class:</b> Keter</p>\n<p><b>Special Containment Procedures:</b> If the sky begins changing color with no prior stimuli or indication, all Foundation sites are to go onto ALERT LEVEL 7 immediately. Personnel at Site-05 and -06 are to enter ALERT LEVEL 8 and begin evaluating new methods of containing SCP-001.</p>\n<p>Site-05 has been constructed around Point Alpha. Procedure 001-ENTRY is in effect to contain SCP-001-1. A monitor containing Document 032, as well as a speaker reading the document, are to be placed at least five meters away from SCP-001-1. If SCP-001-1 swells up to 2 meters or more in diameter, the speed of the narration is to increase. This system is to have a direct cable link to the Foundation SCiPNET database at Site-06, and several redundant power generators are to ensure this system stays operational at all times.</p>\n<p><b>Description:</b> SCP-001 is an ongoing ZK-Class "Reality Failure" Scenario. SCP-001 is prevented from happening through several procedures manipulating its sub-anomalies. The only known observable symptom of SCP-001 is the sky becoming a different color; it is believed that, if SCP-001 were to progress to the point where other symptoms began to occur, it would become irreversible.</p>\n<p>SCP-001-1 is a levitating object contained within Point Alpha, a cave chamber five kilometers south of the ruins of Babylon. SCP-001-1 visually resembles a perfectly smooth sphere with a texture similar to blurred television static. Measurement instruments indicate SCP-001-1 is currently <span class="rndiam"></span> meters in diameter. Solids and liquids that pass within the bounds of SCP-001-1 are replaced entirely with argon gas, giving the illusion of disappearing.</p>\n<p>SCP-001-2 is an entity that is capable of being seen within Point Alpha. SCP-001-2's manifestations are random; witnesses report seeing SCP-001-2 "past the walls of the cave." From descriptions of eyewitnesses, SCP-001-2 is humanoid in shape and large in stature, and is only visible when looking through SCP-001-1. Descriptions of SCP-001-2 vary from person to person; common features include an emaciated figure, long limbs, and prominent facial features. SCP-001-2 takes the position of lying against the wall. It is most often seen with a neutral expression; however, the expression occasionally changes to discomfort or bliss.</p>\n<p>It has been determined from historical precedent that "entertaining" SCP-001-2 through SCP-001-1 will prevent SCP-001.</p>\n<div class="scp-image-block block-left" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/cave.jpg" style="width:300px;" alt="cave.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Point Alpha shortly after it was acquired by the Foundation, with Agents Samuel and Boise preforming preliminary evaluation. SCP-001-1 not visible.</p>\n</div>\n</div>\n<p><b>History:</b> Point Alpha and its contents are believed to date back to prehistory. Babylonian scripts indicate that SCP-001-1 was routinely patrolled by a group of sages, known as the "Order of White and Black," who refused passage to anyone who could demonstrate the ability to read or write.</p>\n<p>This group was made defunct by the rise of Achaemenid Persia, which contained SCP-001-1 with a group of Zoroastrian monks. Very few accounts of SCP-001-1 are present; recovered accounts assert that all information about SCP-001-1 and its significance was passed orally.</p>\n<p>The most substantial account of SCP-001-1 and SCP-001-2 at this time dates back to the Greek philosopher Xera, who made an expedition into the Achaemenid Empire and found Point Alpha. During Alexander of Macedon's conquest of the Achaemenid Empire, he took interest in Xera's texts and continued the Achaemenid containment of SCP-001-1. However, this new institution, known as the "Cronus Guard", was given Greek epics to read in order to "punish Cronus" for his actions. This institution lasted through the partition of Macedon into the Seleucid Empire, the rise of the Sassanid Empire, and even the formation of the Rashidun Caliphate.</p>\n<p>The Cronus Guard were eventually replaced with an Islamic equivalent: "The Society for the Containment of the Babel Demon." This group was the first to preform substantial research on SCP-001-1, and the first to accurately link SCP-001 to SCP-001-1.</p>\n<div class="socotbd">\n<p style="font-size: 82%"><a onclick="toggle('socotbd');">View Attachment: Excerpts from the notes of the Society for the Containment of the Babel Demon</a></p>\n</div>\n<div class="socotbd collapsed">\n <p style="font-size: 82%"><a onclick="toggle('socotbd');">\nClose Attachment</a></p>\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/book.jpg" style="width:300px;" alt="book.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Records from the Society for the Containment of the Babel Demon, recovered in the early 19<sup>th</sup> century within ORIA archives.</p>\n</div>\n</div>\n<blockquote>\n<p><b>Literature Read:</b> The first quatrain of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON squirms but otherwise does not react. Consistent with reading of Homer's Epics by the Chronus Guard.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> The remaining quatrains of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> No changes from observed behavior.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> A poem written by Guardian Muhammad ibn Buya'aa meant explicitly for DEMON</p>\n<p><b>Result:</b> DEMON thrashes more than usual until the cessation of the reading. ORB begins to expand until the <i>Genealogies of the Nobles</i> is read to DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> None, for 10 days</p>\n<p><b>Result:</b> ORB is observed to expand, and DEMON is observed to thrash. Externally, the sky above the camp is noted to turn a darker red color, until the <i>Rubaiyat of Omar Khayyam</i> is read again.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON thrashes more than during previous readings of the <i>Rubaiyat of Omar Khayyam</i>. It is believed that the Quran may no longer have any effect on DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>One Thousand and One Nights</i></p>\n<p><b>Result:</b> DEMON ceases thrashing. New literature may be necessary to prevent further phenomena from occurring.</p>\n</blockquote>\n</div>\n<p>The Society was eventually absorbed into the Office for the Reclamation of Islamic Artifacts, who assumed containment of SCP-001-1. After Incident 001-EXAL, the Foundation purchased Point Alpha from ORIA for a large sum of currency and several Safe-class anomalies.</p>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Incident 001-EXAL</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<p style="font-size: 82%"><i>Note: Some documentation taken from ORIA's account of Incident 001-EXAL.</i></p>\n<p>On <span id="today"></span>, personnel with ORIA read the book <i>Tafsir al-Ahlam al-kabir</i>, or "Great Book of Interpretation of Dreams" as per normal containment of SCP-001-1. However, SCP-001-2 was observed to rapidly thrash while screaming. SCP-001-1 itself swelled from 50.3 centimeters to 3.2 meters in diameter.</p>\n<p>Shortly following this event, SCP-001 began to worsen. The sky worldwide turned a black/white color similar to the texture of SCP-001-1. In addition, reality bending phenomena began worldwide, causing deformed geography, the manifestation of dangerous anomalous objects and several natural disasters. This event was ended after ORIA personnel read SCP-001-1 an undisclosed number of as-of-yet unread books, which caused SCP-001-2 to stop thrashing and SCP-001 to restore to its pre-incident point. However, the damage caused by this incident was deemed enough to require an activation of <a target="_top" href="http://www.scp-wiki.net/scp-2000">SCP-2000</a>.</p>\n<p>This prompted the Foundation to take control of containment of SCP-001. See <b>History</b> segment above for more information.</p>\n<div class="info">\n<p style="font-size: 82%"><a onclick="toggle('info');">Input Level 5/001 Credentials</a></p>\n</div>\n<div class="info collapsed">\n <p style="font-size: 82%"><a onclick="toggle('info');">\nAccess Granted</a></p>\n<p>During Incident 001-Alpha, vocalizations in <a target="_top" href="http://www.scp-wiki.net/scp-140">Daevish</a> were recorded by ORIA instruments within Point Alpha. The source is presently unknown. The following are approximate translations of these vocalizations.</p>\n<blockquote>\n<p>It has passed some [space/time].</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. It [has had/used to be] fun. It is [time/space] to leave.</p>\n</blockquote>\n<blockquote>\n<p>You cannot [stay/root/hold] yourself forever.</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. You [must/will] awaken.</p>\n</blockquote>\n<blockquote>\n<p>No, you have [entered/reentered] slumber for too long. Wake up.</p>\n</blockquote>\n<blockquote>\n<p>[King/Prince/loved one], it is time for you to wake up.</p>\n</blockquote>\n<blockquote>\n<p>It is fun to be in the [UNKNOWN: gaera] but you cannot be [in/rooted] there [forever/unending]. It is [entertaining/deathlike], but you must wake up.</p>\n</blockquote>\n<blockquote>\n<p>Wake up, [King/Prince/loved one]. We miss you.</p>\n</blockquote>\n</div>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Addenda</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<div class="discus">\n<p style="font-size: 82%"><a onclick="toggle('discus');">Input Level 5/001 Clearance</a></p>\n</div>\n<div class="discus collapsed">\n <p style="font-size: 82%"><a onclick="toggle('discus');">\nAccess Granted</a></p>\n<div div class="content-panel standalone series">\n<center>\n<h2>Discussion Thread 001-398:<br />Continued Containment</h2>\n<p style="font-size: 82%">Started on: <span id="today2"></span></p>\n<p style="font-size: 82%">Started by: <tt>HMCL Robinson</tt></p>\n</center>\n</div>\n<blockquote>\n<p><b><tt>HMCL Robinson:</tt></b> The usage of this thread will be for containment directives for SCP-001. In the past year that we have spent containing it, SCP-001-1's literature needs have become increasingly draconic: we can very rarely reread books without it beginning to thrash, and even then we're running out of high-quality literature to give it. We need a more long-term, reliable solution for containment. SCP-001's file has been temporarily opened up to all personnel with Level 2 clearance. Anybody can submit an idea.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Use Artificial Intelligence Constructs to automatically generate new stories for SCP-001-2.</p>\n<p><b>Status: <tt>[ <span class="green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> TSATPWTCOTTTADC.aic was able to generate 10,000 volumes of stories imitating Greek literature. However, when the first was read to SCP-001-1, it began thrashing beyond acceptable measures until it was read approved reading. No further computer-generated works are to be given to SCP-001-1.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Hire a full team of authors to create literature for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> In the past, literature written specifically for SCP-001-1 have failed to contain it (see: SCtBD Document #249). With SCP-001-2's current state this is not to be attempted again.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> This could represent a possible information leak. Test denied.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Reason:</b> With SCP-001-2's recent containment breach, these measures are approved.</p>\n<p><b>Result:</b> SCP-001-2 observed to stop thrashing and stay completely still, seemingly smiling. In addition, SCP-███ was able to be read for 14 consecutive readings before SCP-001-2 resumed normal activity.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Regularly read mainlist documentation for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Replacement of the SCP-001 file with several "001 Proposals" with much more grand implications than normal SCP files.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> By only occasionally reading 001 files, SCP-001-2 is observed to be more calm on average. The previously observed actions of retaliation in response to specially crafted literature do not apply here.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Given the variation of the current 2957 SCP mainlist files, SCP-001-1 is to be read a random arrangement of these files, in repeat.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> This strategy succeeded for approximately 3 years, allowing for several repeats. However, on <span id="yearplus3"></span>-9-30, SCP-001-2 began thrashing rapidly, requiring newer documentation to sedate.</p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-4999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> SCP-001-1 has required progressively more SCP mainlist articles for containment, even to the point of the fabrication of some anomalous entities.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Rewriting of several older SCP mainlist files to be more exaggerated and narrative-like.</p>\n<p><b>Status: <tt>[ <span style="color: orange">TENTATIVELY APPROVED</span> ]</tt></b></p>\n<p><b>Note:</b> <i>Deliberately exaggerating our files for the purpose of this entity is crossing some lines, but for now, it will have to do. However, we need a better way. I'm calling a committee. We need to put a stop to this.</i></p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Creation of a file describing SCP-001 that uses recursion to effectively be "never-ending." Psychoanalysis related to SCP-001-2's apparent "enjoyment" of works implies a slight ego and bias towards works about itself, and past strategies with this idea have worked for long periods of time. File is to be archived as a "technical" file.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> Procedure 001-ENTRY implemented. [DATA EXPUNGED]</p>\n</blockquote>\n</div>\n\n<div class="newiter">\n<p style="font-size: 82%"><a onclick="new_iteration();">Input Level 5/TECHNICAL Clearance</a></p>\n</div>\n<div class="inner">\n<!-- :-) -->\n</div>\n</div>\n</body>\n</html>
254	[[>]]\n[[module Rate]]\n[[/>]]\n<iframe>\n<body>\n<style type="text/css">\nbody\n{\n   font-family:verdana,arial,helvetica,sans-serif;\n   font-size:12.8px;\n   color:#333;\n   line-height:141%\n}\nblockquote\n{\n  border:1px dashed #999;\n  padding: 0 12.8px;\n  background-color:#f4f4f4\n}\na {\n    color: #b01;\n    text-decoration: none;\n    background: transparent;\n}\n.content-panel {\n    border: solid 1px #888880;\n    border-radius: 10px;\n    background-color: #999990;\n    margin: 10px 0 15px;\n    box-shadow: 3px 3px 6px #bbb;\n    box-shadow: 0 2px 6px rgba(0,0,0,0.5), inset 0 1px rgba(255,255,255,0.3), inset 0 10px rgba(255,255,255,0.2), inset 0 10px 20px rgba(255,255,255,0.25), inset 0 -15px 30px rgba(0,0,0,0.1);\n}\n.content-panel.standalone {\n    background: #fcfdfb;\n}\n.content-panel.series {\n    padding: 0 20px;\n    margin-bottom: 20px;\n}\na.newpage {\n    color: #d61;\n    text-decoration: none;\n    background: transparent;\n}\na:hover {\n    text-decoration: underline;\n    background-color: transparent;\n}\n.collapsed {\n  display: none;\n}\n.vanished {\n  display: none;\n}\n.warning-cont {\n  color: red;\n  text-align: center;\n}\n#warning-header {\n  font-size: 25.6px;\n}\n#warning {\n  font-size: 20px;\n}\n.retrofont {\nfont-family: 'Geo', cursive;\nfont-weight: bold;\n}\n.cblwarning {\n  color: red;\n}\n.green {\n  color: green;\n}\n.smaller-g {\nwidth: 45%;\nmargin: auto;\nbackground-color: #e6e6e6;\nborder: none;\nheight: 2px;\n}\n.adden{\nfont-size: 1.3em;\n}\n/* Standard Image Block */\n.scp-image-block {\n    border: solid 1px #666;\n    box-shadow: 0 1px 6px rgba(0,0,0,.25);\n    width: 300px;\n}\n \n.scp-image-block.block-right {\n        float: right;\n    clear: right;\n    margin: 0 2em 1em 2em;\n}\n \n.scp-image-block.block-left {\n    float: left;\n    clear: left;\n    margin: 0 2em 1em 0;\n}\n \n.scp-image-block.block-center {\n    margin-right: auto;\n    margin-left: auto;\n}\n.scp-image-block img {\n    border: 0;\n    width: 300px;\n}\n.scp-image-block .scp-image-caption {\n    background-color: #eee;\n    border-top: solid 1px #666;\n    padding: 2px 0;\n    font-size: 80%;\n    font-weight: bold;\n    text-align: center;\n    width: 300px;\n}\n.scp-image-block > p {\n    margin: 0;\n}\n.scp-image-block .scp-image-caption > p {\n    margin: 0;\n    padding: 0 10px;\n    line-height: 125%\n}\n</style>\n<script type="text/javascript">\nfunction addEvent(element, eventName, callback) {\n    if (element.addEventListener) {\n        element.addEventListener(eventName, callback, false);\n    } else if (element.attachEvent) {\n        element.attachEvent("on" + eventName, callback);\n    } else {\n        element["on" + eventName] = callback;\n    }\n}\n\nwindow.toggle = function(cls) {\n  var collapsed = document.querySelectorAll("." + cls + ".collapsed");\n  var expanded = document.querySelectorAll("." + cls + ":not(.collapsed)");\n  var i;\n  for(i = 0; i < collapsed.length; i++) {\n    collapsed[i].classList.remove("collapsed");\n  }\n  for(i = 0; i < expanded.length; i++) {\n    expanded[i].classList.add("collapsed");\n  }\n};\n\n// typewriter code\nfunction Typewriter(el, str, delay) {\n  if (!(this instanceof Typewriter)) return new Typewriter(el, str, delay);\n\n  this.el = el;\n  this.str = str || el.innerHTML || el.value;\n  this.delay = delay || 100;\n  this.i = 0;\n}\n\nTypewriter.prototype.type = function() {\n  var i = this.i,\n    char = this.str.charAt(i);\n\n  if (!char) {\n    clearInterval(this.intervalID);\n    return this;\n  }\n\n  if (char === '<') this.isTag = true;\n  if (char === '>') this.isTag = false;\n\n  this.el.innerHTML += char;\n  return this.i++;\n};\n\nTypewriter.prototype.start = function() {\n  var self = this;\n  if (this.i < 0) this.i = 0;\n  if (this.el.innerHTML === this.str) this.clear();\n\n  // this.emit('start');\n  (function loop() {\n    self.type();\n    if (self.isTag) return loop();\n    self.intervalID = setTimeout(loop, self.delay);\n  }());\n\n  return this;\n};\n\nTypewriter.prototype.stop = function() {\n  this.i = -1;\n  return this;\n};\n\nTypewriter.prototype.restart = function() {\n  this.clear();\n  this.i = 0;\n  return this.start();\n};\n\nTypewriter.prototype.clear = function() {\n  this.el.innerHTML = '';\n  return this;\n};\n\nvar junkChars = ['ó','ò','ñ','ç','¿','ß','à','è','ì','ù','Œ','þ','█','%','+','=','.','«','»','£','$','€','½','¾','À','È','Ì','Ò','Ù','Â','Ê','â','ê','Ä','Ë','ä','ë','ÿ','ü','•'];\n\nvar an_iteration = "";\n\n\nvar randomIntFromInterval = function(min,max)\n{\n  return Math.floor(Math.random()*(max-min+1)+min);\n};\n\nvar getJunkChars = function(length) {\n  var res = '';\n  var i;\n  for (i = 0; i < length; i++) {\n    res += junkChars[randomIntFromInterval(0,junkChars.length - 1)];\n  }\n  return res;\n};\n\nvar selTabs = [];\n\nfunction rndiam() {\n  var rndiams = document.getElementsByClassName('rndiam');\n  var i;\n  for (i = 0; i < rndiams.length; i++) {\n    rndiams[i].innerHTML = (Math.random() * (1.56 - 0.51) + 0.51).toFixed(2);\n  }\n  setTimeout(rndiam, 2000);\n}\n\nwindow.onload = function() {\n\nselTabs = [\n  document.getElementsByClassName('selTab1')[0],\n  document.getElementsByClassName('selTab2')[0],\n  document.getElementsByClassName('selTab3')[0],\n  document.getElementsByClassName('selTab4')[0],\n  document.getElementsByClassName('selTab5')[0]\n];\n\nvar msg1 = '= SCP-001 | Technical Clearance Required =';\n\nvar delay1 = 50;\nvar delay2 = 25;\n\nvar toD1 = delay1 * (msg1.length);\n\nvar stopEarly = function(){\n  document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished");\n};\n\nvar typ1 =Typewriter(document.querySelectorAll('#warning-header')[0],msg1,delay1)\n\ntyp1.start();\nsetTimeout(function() { document.getElementById("warning").classList.remove("vanished"); }, toD1 + 500);\nsetTimeout(function () { document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished"); }, toD1 + 1500);\n\n  rndiam();\n\n  var today = new Date();\n  var todayformat = "" + (today.getFullYear() - 1) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today").innerHTML = todayformat;\n  var todayformat2 = "" + (today.getFullYear() - 0) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today2").innerHTML = todayformat2;\n  document.getElementById("yearplus3").innerHTML = "" + (today.getFullYear() + 3);\n\n  an_iteration = document.getElementById("part1").innerHTML;\n};\n\nwindow.new_iteration = function() {\n  var new_iters = document.getElementsByClassName("newiter");\n  console.log(new_iters);\n  new_iters[new_iters.length - 1].classList.add("vanished");\n  var inner = document.getElementsByClassName("inner");\n  inner[inner.length - 1].innerHTML = "<hr /><hr />" + an_iteration;\n};\n</script>\n</script>\n<div class="warning-cont">\n<h2 id="warning-header">\n<p>\n</p>\n<h3 id="warning" class="vanished">  \n<p>This document exists as technical containment for an anomaly and thus does not adhere to standard SCiPNET formatting. Input 5/TECHNICAL clearance to proceed.\n</p>\n</div>\n\n<div id="collapsible_to_open_skip" class="article vanished">\n<p><center><a onclick="toggle('article');">[ INPUT PERSONAL IDENTIFICATION NUMBER ]</a></center></p>\n</div>\n<div class="article collapsed">\n <p><center><a onclick="toggle('article');">[ PERSONAL IDENTIFICATION NUMBER VERIFIED ]</a></center></p>\n<div id="part1">\n<hr />\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://topia.wikidot.com/local--files/jamcon-001/sky.jpg" style="width:300px;" alt="destruction.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>The sky above Point Alpha during a breach of SCP-001.</p>\n</div>\n</div>\n<p><b>Item #:</b> SCP-001</p>\n<p><b>Object Class:</b> Ontokinetic</p>\n<p><b>Containment Class:</b> Keter</p>\n<p><b>Special Containment Procedures:</b> If the sky begins changing color with no prior stimuli or indication, all Foundation sites are to go onto ALERT LEVEL 7 immediately. Personnel at Site-05 and -06 are to enter ALERT LEVEL 8 and begin evaluating new methods of containing SCP-001.</p>\n<p>Site-05 has been constructed around Point Alpha. Procedure 001-ENTRY is in effect to contain SCP-001-1. A monitor containing Document 032, as well as a speaker reading the document, are to be placed at least five meters away from SCP-001-1. If SCP-001-1 swells up to 2 meters or more in diameter, the speed of the narration is to increase. This system is to have a direct cable link to the Foundation SCiPNET database at Site-06, and several redundant power generators are to ensure this system stays operational at all times.</p>\n<p><b>Description:</b> SCP-001 is an ongoing ZK-Class "Reality Failure" Scenario. SCP-001 is prevented from happening through several procedures manipulating its sub-anomalies. The only known observable symptom of SCP-001 is the sky becoming a different color; it is believed that, if SCP-001 were to progress to the point where other symptoms began to occur, it would become irreversible.</p>\n<p>SCP-001-1 is a levitating object contained within Point Alpha, a cave chamber five kilometers south of the ruins of Babylon. SCP-001-1 visually resembles a perfectly smooth sphere with a texture similar to blurred television static. Measurement instruments indicate SCP-001-1 is currently <span class="rndiam"></span> meters in diameter. Solids and liquids that pass within the bounds of SCP-001-1 are replaced entirely with argon gas, giving the illusion of disappearing.</p>\n<p>SCP-001-2 is an entity that is capable of being seen within Point Alpha. SCP-001-2's manifestations are random; witnesses report seeing SCP-001-2 "past the walls of the cave." From descriptions of eyewitnesses, SCP-001-2 is humanoid in shape and large in stature, and is only visible when looking through SCP-001-1. Descriptions of SCP-001-2 vary from person to person; common features include an emaciated figure, long limbs, and prominent facial features. SCP-001-2 takes the position of lying against the wall. It is most often seen with a neutral expression; however, the expression occasionally changes to discomfort or bliss.</p>\n<p>It has been determined from historical precedent that "entertaining" SCP-001-2 through SCP-001-1 will prevent SCP-001.</p>\n<div class="scp-image-block block-left" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/cave.jpg" style="width:300px;" alt="cave.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Point Alpha shortly after it was acquired by the Foundation, with Agents Samuel and Boise preforming preliminary evaluation. SCP-001-1 not visible.</p>\n</div>\n</div>\n<p><b>History:</b> Point Alpha and its contents are believed to date back to prehistory. Babylonian scripts indicate that SCP-001-1 was routinely patrolled by a group of sages, known as the "Order of White and Black," who refused passage to anyone who could demonstrate the ability to read or write.</p>\n<p>This group was made defunct by the rise of Achaemenid Persia, which contained SCP-001-1 with a group of Zoroastrian monks. Very few accounts of SCP-001-1 are present; recovered accounts assert that all information about SCP-001-1 and its significance was passed orally.</p>\n<p>The most substantial account of SCP-001-1 and SCP-001-2 at this time dates back to the Greek philosopher Xera, who made an expedition into the Achaemenid Empire and found Point Alpha. During Alexander of Macedon's conquest of the Achaemenid Empire, he took interest in Xera's texts and continued the Achaemenid containment of SCP-001-1. However, this new institution, known as the "Cronus Guard", was given Greek epics to read in order to "punish Cronus" for his actions. This institution lasted through the partition of Macedon into the Seleucid Empire, the rise of the Sassanid Empire, and even the formation of the Rashidun Caliphate.</p>\n<p>The Cronus Guard were eventually replaced with an Islamic equivalent: "The Society for the Containment of the Babel Demon." This group was the first to preform substantial research on SCP-001-1, and the first to accurately link SCP-001 to SCP-001-1.</p>\n<div class="socotbd">\n<p style="font-size: 82%"><a onclick="toggle('socotbd');">View Attachment: Excerpts from the notes of the Society for the Containment of the Babel Demon</a></p>\n</div>\n<div class="socotbd collapsed">\n <p style="font-size: 82%"><a onclick="toggle('socotbd');">\nClose Attachment</a></p>\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/book.jpg" style="width:300px;" alt="book.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Records from the Society for the Containment of the Babel Demon, recovered in the early 19<sup>th</sup> century within ORIA archives.</p>\n</div>\n</div>\n<blockquote>\n<p><b>Literature Read:</b> The first quatrain of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON squirms but otherwise does not react. Consistent with reading of Homer's Epics by the Chronus Guard.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> The remaining quatrains of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> No changes from observed behavior.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> A poem written by Guardian Muhammad ibn Buya'aa meant explicitly for DEMON</p>\n<p><b>Result:</b> DEMON thrashes more than usual until the cessation of the reading. ORB begins to expand until the <i>Genealogies of the Nobles</i> is read to DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> None, for 10 days</p>\n<p><b>Result:</b> ORB is observed to expand, and DEMON is observed to thrash. Externally, the sky above the camp is noted to turn a darker red color, until the <i>Rubaiyat of Omar Khayyam</i> is read again.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON thrashes more than during previous readings of the <i>Rubaiyat of Omar Khayyam</i>. It is believed that the Quran may no longer have any effect on DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>One Thousand and One Nights</i></p>\n<p><b>Result:</b> DEMON ceases thrashing. New literature may be necessary to prevent further phenomena from occurring.</p>\n</blockquote>\n</div>\n<p>The Society was eventually absorbed into the Office for the Reclamation of Islamic Artifacts, who assumed containment of SCP-001-1. After Incident 001-EXAL, the Foundation purchased Point Alpha from ORIA for a large sum of currency and several Safe-class anomalies.</p>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Incident 001-EXAL</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<p style="font-size: 82%"><i>Note: Some documentation taken from ORIA's account of Incident 001-EXAL.</i></p>\n<p>On <span id="today"></span>, personnel with ORIA read the book <i>Tafsir al-Ahlam al-kabir</i>, or "Great Book of Interpretation of Dreams" as per normal containment of SCP-001-1. However, SCP-001-2 was observed to rapidly thrash while screaming. SCP-001-1 itself swelled from 50.3 centimeters to 3.2 meters in diameter.</p>\n<p>Shortly following this event, SCP-001 began to worsen. The sky worldwide turned a black/white color similar to the texture of SCP-001-1. In addition, reality bending phenomena began worldwide, causing deformed geography, the manifestation of dangerous anomalous objects and several natural disasters. This event was ended after ORIA personnel read SCP-001-1 an undisclosed number of as-of-yet unread books, which caused SCP-001-2 to stop thrashing and SCP-001 to restore to its pre-incident point. However, the damage caused by this incident was deemed enough to require an activation of <a target="_top" href="http://www.scp-wiki.net/scp-2000">SCP-2000</a>.</p>\n<p>This prompted the Foundation to take control of containment of SCP-001. See <b>History</b> segment above for more information.</p>\n<div class="info">\n<p style="font-size: 82%"><a onclick="toggle('info');">Input Level 5/001 Credentials</a></p>\n</div>\n<div class="info collapsed">\n <p style="font-size: 82%"><a onclick="toggle('info');">\nAccess Granted</a></p>\n<p>During Incident 001-Alpha, vocalizations in <a target="_top" href="http://www.scp-wiki.net/scp-140">Daevish</a> were recorded by ORIA instruments within Point Alpha. The source is presently unknown. The following are approximate translations of these vocalizations.</p>\n<blockquote>\n<p>It has passed some [space/time].</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. It [has had/used to be] fun. It is [time/space] to leave.</p>\n</blockquote>\n<blockquote>\n<p>You cannot [stay/root/hold] yourself forever.</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. You [must/will] awaken.</p>\n</blockquote>\n<blockquote>\n<p>No, you have [entered/reentered] slumber for too long. Wake up.</p>\n</blockquote>\n<blockquote>\n<p>[King/Prince/loved one], it is time for you to wake up.</p>\n</blockquote>\n<blockquote>\n<p>It is fun to be in the [UNKNOWN: gaera] but you cannot be [in/rooted] there [forever/unending]. It is [entertaining/deathlike], but you must wake up.</p>\n</blockquote>\n<blockquote>\n<p>Wake up, [King/Prince/loved one]. We miss you.</p>\n</blockquote>\n</div>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Addenda</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<div class="discus">\n<p style="font-size: 82%"><a onclick="toggle('discus');">Input Level 5/001 Clearance</a></p>\n</div>\n<div class="discus collapsed">\n <p style="font-size: 82%"><a onclick="toggle('discus');">\nAccess Granted</a></p>\n<div div class="content-panel standalone series">\n<center>\n<h2>Discussion Thread 001-398:<br />Continued Containment</h2>\n<p style="font-size: 82%">Started on: <span id="today2"></span></p>\n<p style="font-size: 82%">Started by: <tt>HMCL Robinson</tt></p>\n</center>\n</div>\n<blockquote>\n<p><b><tt>HMCL Robinson:</tt></b> The usage of this thread will be for containment directives for SCP-001. In the past year that we have spent containing it, SCP-001-1's literature needs have become increasingly draconic: we can very rarely reread books without it beginning to thrash, and even then we're running out of high-quality literature to give it. We need a more long-term, reliable solution for containment. SCP-001's file has been temporarily opened up to all personnel with Level 2 clearance. Anybody can submit an idea.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Use Artificial Intelligence Constructs to automatically generate new stories for SCP-001-2.</p>\n<p><b>Status: <tt>[ <span class="green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> TSATPWTCOTTTADC.aic was able to generate 10,000 volumes of stories imitating Greek literature. However, when the first was read to SCP-001-1, it began thrashing beyond acceptable measures until it was read approved reading. No further computer-generated works are to be given to SCP-001-1.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Hire a full team of authors to create literature for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> In the past, literature written specifically for SCP-001-1 have failed to contain it (see: SCtBD Document #249). With SCP-001-2's current state this is not to be attempted again.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> This could represent a possible information leak. Test denied.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Reason:</b> With SCP-001-2's recent containment breach, these measures are approved.</p>\n<p><b>Result:</b> SCP-001-2 observed to stop thrashing and stay completely still, seemingly smiling. In addition, SCP-███ was able to be read for 14 consecutive readings before SCP-001-2 resumed normal activity.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Regularly read mainlist documentation for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Replacement of the SCP-001 file with several "001 Proposals" with much more grand implications than normal SCP files.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> By only occasionally reading 001 files, SCP-001-2 is observed to be more calm on average. The previously observed actions of retaliation in response to specially crafted literature do not apply here.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Given the variation of the current 2957 SCP mainlist files, SCP-001-1 is to be read a random arrangement of these files, in repeat.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> This strategy succeeded for approximately 3 years, allowing for several repeats. However, on <span id="yearplus3"></span>-9-30, SCP-001-2 began thrashing rapidly, requiring newer documentation to sedate.</p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-4999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> SCP-001-1 has required progressively more SCP mainlist articles for containment, even to the point of the fabrication of some anomalous entities.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Rewriting of several older SCP mainlist files to be more exaggerated and narrative-like.</p>\n<p><b>Status: <tt>[ <span style="color: orange">TENTATIVELY APPROVED</span> ]</tt></b></p>\n<p><b>Note:</b> <i>Deliberately exaggerating our files for the purpose of this entity is crossing some lines, but for now, it will have to do. However, we need a better way. I'm calling a committee. We need to put a stop to this.</i></p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Creation of a file describing SCP-001 that uses recursion to effectively be "never-ending." Psychoanalysis related to SCP-001-2's apparent "enjoyment" of works implies a slight ego and bias towards works about itself, and past strategies with this idea have worked for long periods of time. File is to be archived as a "technical" file.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> Procedure 001-ENTRY implemented. [DATA EXPUNGED]</p>\n</blockquote>\n</div>\n\n<div class="newiter">\n<p style="font-size: 82%"><a onclick="new_iteration();">Input Level 5/TECHNICAL Clearance</a></p>\n</div>\n<div class="inner">\n<!-- :-) -->\n</div>\n</div>\n</body>\n</iframe>
255	[[>]]\n[[module Rate]]\n[[/>]]\n<html>\n<body>\n<style type="text/css">\nbody\n{\n   font-family:verdana,arial,helvetica,sans-serif;\n   font-size:12.8px;\n   color:#333;\n   line-height:141%\n}\nblockquote\n{\n  border:1px dashed #999;\n  padding: 0 12.8px;\n  background-color:#f4f4f4\n}\na {\n    color: #b01;\n    text-decoration: none;\n    background: transparent;\n}\n.content-panel {\n    border: solid 1px #888880;\n    border-radius: 10px;\n    background-color: #999990;\n    margin: 10px 0 15px;\n    box-shadow: 3px 3px 6px #bbb;\n    box-shadow: 0 2px 6px rgba(0,0,0,0.5), inset 0 1px rgba(255,255,255,0.3), inset 0 10px rgba(255,255,255,0.2), inset 0 10px 20px rgba(255,255,255,0.25), inset 0 -15px 30px rgba(0,0,0,0.1);\n}\n.content-panel.standalone {\n    background: #fcfdfb;\n}\n.content-panel.series {\n    padding: 0 20px;\n    margin-bottom: 20px;\n}\na.newpage {\n    color: #d61;\n    text-decoration: none;\n    background: transparent;\n}\na:hover {\n    text-decoration: underline;\n    background-color: transparent;\n}\n.collapsed {\n  display: none;\n}\n.vanished {\n  display: none;\n}\n.warning-cont {\n  color: red;\n  text-align: center;\n}\n#warning-header {\n  font-size: 25.6px;\n}\n#warning {\n  font-size: 20px;\n}\n.retrofont {\nfont-family: 'Geo', cursive;\nfont-weight: bold;\n}\n.cblwarning {\n  color: red;\n}\n.green {\n  color: green;\n}\n.smaller-g {\nwidth: 45%;\nmargin: auto;\nbackground-color: #e6e6e6;\nborder: none;\nheight: 2px;\n}\n.adden{\nfont-size: 1.3em;\n}\n/* Standard Image Block */\n.scp-image-block {\n    border: solid 1px #666;\n    box-shadow: 0 1px 6px rgba(0,0,0,.25);\n    width: 300px;\n}\n \n.scp-image-block.block-right {\n        float: right;\n    clear: right;\n    margin: 0 2em 1em 2em;\n}\n \n.scp-image-block.block-left {\n    float: left;\n    clear: left;\n    margin: 0 2em 1em 0;\n}\n \n.scp-image-block.block-center {\n    margin-right: auto;\n    margin-left: auto;\n}\n.scp-image-block img {\n    border: 0;\n    width: 300px;\n}\n.scp-image-block .scp-image-caption {\n    background-color: #eee;\n    border-top: solid 1px #666;\n    padding: 2px 0;\n    font-size: 80%;\n    font-weight: bold;\n    text-align: center;\n    width: 300px;\n}\n.scp-image-block > p {\n    margin: 0;\n}\n.scp-image-block .scp-image-caption > p {\n    margin: 0;\n    padding: 0 10px;\n    line-height: 125%\n}\n</style>\n<script type="text/javascript">\nfunction addEvent(element, eventName, callback) {\n    if (element.addEventListener) {\n        element.addEventListener(eventName, callback, false);\n    } else if (element.attachEvent) {\n        element.attachEvent("on" + eventName, callback);\n    } else {\n        element["on" + eventName] = callback;\n    }\n}\n\nwindow.toggle = function(cls) {\n  var collapsed = document.querySelectorAll("." + cls + ".collapsed");\n  var expanded = document.querySelectorAll("." + cls + ":not(.collapsed)");\n  var i;\n  for(i = 0; i < collapsed.length; i++) {\n    collapsed[i].classList.remove("collapsed");\n  }\n  for(i = 0; i < expanded.length; i++) {\n    expanded[i].classList.add("collapsed");\n  }\n};\n\n// typewriter code\nfunction Typewriter(el, str, delay) {\n  if (!(this instanceof Typewriter)) return new Typewriter(el, str, delay);\n\n  this.el = el;\n  this.str = str || el.innerHTML || el.value;\n  this.delay = delay || 100;\n  this.i = 0;\n}\n\nTypewriter.prototype.type = function() {\n  var i = this.i,\n    char = this.str.charAt(i);\n\n  if (!char) {\n    clearInterval(this.intervalID);\n    return this;\n  }\n\n  if (char === '<') this.isTag = true;\n  if (char === '>') this.isTag = false;\n\n  this.el.innerHTML += char;\n  return this.i++;\n};\n\nTypewriter.prototype.start = function() {\n  var self = this;\n  if (this.i < 0) this.i = 0;\n  if (this.el.innerHTML === this.str) this.clear();\n\n  // this.emit('start');\n  (function loop() {\n    self.type();\n    if (self.isTag) return loop();\n    self.intervalID = setTimeout(loop, self.delay);\n  }());\n\n  return this;\n};\n\nTypewriter.prototype.stop = function() {\n  this.i = -1;\n  return this;\n};\n\nTypewriter.prototype.restart = function() {\n  this.clear();\n  this.i = 0;\n  return this.start();\n};\n\nTypewriter.prototype.clear = function() {\n  this.el.innerHTML = '';\n  return this;\n};\n\nvar junkChars = ['ó','ò','ñ','ç','¿','ß','à','è','ì','ù','Œ','þ','█','%','+','=','.','«','»','£','$','€','½','¾','À','È','Ì','Ò','Ù','Â','Ê','â','ê','Ä','Ë','ä','ë','ÿ','ü','•'];\n\nvar an_iteration = "";\n\n\nvar randomIntFromInterval = function(min,max)\n{\n  return Math.floor(Math.random()*(max-min+1)+min);\n};\n\nvar getJunkChars = function(length) {\n  var res = '';\n  var i;\n  for (i = 0; i < length; i++) {\n    res += junkChars[randomIntFromInterval(0,junkChars.length - 1)];\n  }\n  return res;\n};\n\nvar selTabs = [];\n\nfunction rndiam() {\n  var rndiams = document.getElementsByClassName('rndiam');\n  var i;\n  for (i = 0; i < rndiams.length; i++) {\n    rndiams[i].innerHTML = (Math.random() * (1.56 - 0.51) + 0.51).toFixed(2);\n  }\n  setTimeout(rndiam, 2000);\n}\n\nwindow.onload = function() {\n\nselTabs = [\n  document.getElementsByClassName('selTab1')[0],\n  document.getElementsByClassName('selTab2')[0],\n  document.getElementsByClassName('selTab3')[0],\n  document.getElementsByClassName('selTab4')[0],\n  document.getElementsByClassName('selTab5')[0]\n];\n\nvar msg1 = '= SCP-001 | Technical Clearance Required =';\n\nvar delay1 = 50;\nvar delay2 = 25;\n\nvar toD1 = delay1 * (msg1.length);\n\nvar stopEarly = function(){\n  document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished");\n};\n\nvar typ1 =Typewriter(document.querySelectorAll('#warning-header')[0],msg1,delay1)\n\ntyp1.start();\nsetTimeout(function() { document.getElementById("warning").classList.remove("vanished"); }, toD1 + 500);\nsetTimeout(function () { document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished"); }, toD1 + 1500);\n\n  rndiam();\n\n  var today = new Date();\n  var todayformat = "" + (today.getFullYear() - 1) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today").innerHTML = todayformat;\n  var todayformat2 = "" + (today.getFullYear() - 0) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today2").innerHTML = todayformat2;\n  document.getElementById("yearplus3").innerHTML = "" + (today.getFullYear() + 3);\n\n  an_iteration = document.getElementById("part1").innerHTML;\n};\n\nwindow.new_iteration = function() {\n  var new_iters = document.getElementsByClassName("newiter");\n  console.log(new_iters);\n  new_iters[new_iters.length - 1].classList.add("vanished");\n  var inner = document.getElementsByClassName("inner");\n  inner[inner.length - 1].innerHTML = "<hr /><hr />" + an_iteration;\n};\n</script>\n</script>\n<div class="warning-cont">\n<h2 id="warning-header">\n<p>\n</p>\n<h3 id="warning" class="vanished">  \n<p>This document exists as technical containment for an anomaly and thus does not adhere to standard SCiPNET formatting. Input 5/TECHNICAL clearance to proceed.\n</p>\n</div>\n\n<div id="collapsible_to_open_skip" class="article vanished">\n<p><center><a onclick="toggle('article');">[ INPUT PERSONAL IDENTIFICATION NUMBER ]</a></center></p>\n</div>\n<div class="article collapsed">\n <p><center><a onclick="toggle('article');">[ PERSONAL IDENTIFICATION NUMBER VERIFIED ]</a></center></p>\n<div id="part1">\n<hr />\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://topia.wikidot.com/local--files/jamcon-001/sky.jpg" style="width:300px;" alt="destruction.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>The sky above Point Alpha during a breach of SCP-001.</p>\n</div>\n</div>\n<p><b>Item #:</b> SCP-001</p>\n<p><b>Object Class:</b> Ontokinetic</p>\n<p><b>Containment Class:</b> Keter</p>\n<p><b>Special Containment Procedures:</b> If the sky begins changing color with no prior stimuli or indication, all Foundation sites are to go onto ALERT LEVEL 7 immediately. Personnel at Site-05 and -06 are to enter ALERT LEVEL 8 and begin evaluating new methods of containing SCP-001.</p>\n<p>Site-05 has been constructed around Point Alpha. Procedure 001-ENTRY is in effect to contain SCP-001-1. A monitor containing Document 032, as well as a speaker reading the document, are to be placed at least five meters away from SCP-001-1. If SCP-001-1 swells up to 2 meters or more in diameter, the speed of the narration is to increase. This system is to have a direct cable link to the Foundation SCiPNET database at Site-06, and several redundant power generators are to ensure this system stays operational at all times.</p>\n<p><b>Description:</b> SCP-001 is an ongoing ZK-Class "Reality Failure" Scenario. SCP-001 is prevented from happening through several procedures manipulating its sub-anomalies. The only known observable symptom of SCP-001 is the sky becoming a different color; it is believed that, if SCP-001 were to progress to the point where other symptoms began to occur, it would become irreversible.</p>\n<p>SCP-001-1 is a levitating object contained within Point Alpha, a cave chamber five kilometers south of the ruins of Babylon. SCP-001-1 visually resembles a perfectly smooth sphere with a texture similar to blurred television static. Measurement instruments indicate SCP-001-1 is currently <span class="rndiam"></span> meters in diameter. Solids and liquids that pass within the bounds of SCP-001-1 are replaced entirely with argon gas, giving the illusion of disappearing.</p>\n<p>SCP-001-2 is an entity that is capable of being seen within Point Alpha. SCP-001-2's manifestations are random; witnesses report seeing SCP-001-2 "past the walls of the cave." From descriptions of eyewitnesses, SCP-001-2 is humanoid in shape and large in stature, and is only visible when looking through SCP-001-1. Descriptions of SCP-001-2 vary from person to person; common features include an emaciated figure, long limbs, and prominent facial features. SCP-001-2 takes the position of lying against the wall. It is most often seen with a neutral expression; however, the expression occasionally changes to discomfort or bliss.</p>\n<p>It has been determined from historical precedent that "entertaining" SCP-001-2 through SCP-001-1 will prevent SCP-001.</p>\n<div class="scp-image-block block-left" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/cave.jpg" style="width:300px;" alt="cave.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Point Alpha shortly after it was acquired by the Foundation, with Agents Samuel and Boise preforming preliminary evaluation. SCP-001-1 not visible.</p>\n</div>\n</div>\n<p><b>History:</b> Point Alpha and its contents are believed to date back to prehistory. Babylonian scripts indicate that SCP-001-1 was routinely patrolled by a group of sages, known as the "Order of White and Black," who refused passage to anyone who could demonstrate the ability to read or write.</p>\n<p>This group was made defunct by the rise of Achaemenid Persia, which contained SCP-001-1 with a group of Zoroastrian monks. Very few accounts of SCP-001-1 are present; recovered accounts assert that all information about SCP-001-1 and its significance was passed orally.</p>\n<p>The most substantial account of SCP-001-1 and SCP-001-2 at this time dates back to the Greek philosopher Xera, who made an expedition into the Achaemenid Empire and found Point Alpha. During Alexander of Macedon's conquest of the Achaemenid Empire, he took interest in Xera's texts and continued the Achaemenid containment of SCP-001-1. However, this new institution, known as the "Cronus Guard", was given Greek epics to read in order to "punish Cronus" for his actions. This institution lasted through the partition of Macedon into the Seleucid Empire, the rise of the Sassanid Empire, and even the formation of the Rashidun Caliphate.</p>\n<p>The Cronus Guard were eventually replaced with an Islamic equivalent: "The Society for the Containment of the Babel Demon." This group was the first to preform substantial research on SCP-001-1, and the first to accurately link SCP-001 to SCP-001-1.</p>\n<div class="socotbd">\n<p style="font-size: 82%"><a onclick="toggle('socotbd');">View Attachment: Excerpts from the notes of the Society for the Containment of the Babel Demon</a></p>\n</div>\n<div class="socotbd collapsed">\n <p style="font-size: 82%"><a onclick="toggle('socotbd');">\nClose Attachment</a></p>\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/book.jpg" style="width:300px;" alt="book.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Records from the Society for the Containment of the Babel Demon, recovered in the early 19<sup>th</sup> century within ORIA archives.</p>\n</div>\n</div>\n<blockquote>\n<p><b>Literature Read:</b> The first quatrain of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON squirms but otherwise does not react. Consistent with reading of Homer's Epics by the Chronus Guard.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> The remaining quatrains of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> No changes from observed behavior.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> A poem written by Guardian Muhammad ibn Buya'aa meant explicitly for DEMON</p>\n<p><b>Result:</b> DEMON thrashes more than usual until the cessation of the reading. ORB begins to expand until the <i>Genealogies of the Nobles</i> is read to DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> None, for 10 days</p>\n<p><b>Result:</b> ORB is observed to expand, and DEMON is observed to thrash. Externally, the sky above the camp is noted to turn a darker red color, until the <i>Rubaiyat of Omar Khayyam</i> is read again.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON thrashes more than during previous readings of the <i>Rubaiyat of Omar Khayyam</i>. It is believed that the Quran may no longer have any effect on DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>One Thousand and One Nights</i></p>\n<p><b>Result:</b> DEMON ceases thrashing. New literature may be necessary to prevent further phenomena from occurring.</p>\n</blockquote>\n</div>\n<p>The Society was eventually absorbed into the Office for the Reclamation of Islamic Artifacts, who assumed containment of SCP-001-1. After Incident 001-EXAL, the Foundation purchased Point Alpha from ORIA for a large sum of currency and several Safe-class anomalies.</p>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Incident 001-EXAL</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<p style="font-size: 82%"><i>Note: Some documentation taken from ORIA's account of Incident 001-EXAL.</i></p>\n<p>On <span id="today"></span>, personnel with ORIA read the book <i>Tafsir al-Ahlam al-kabir</i>, or "Great Book of Interpretation of Dreams" as per normal containment of SCP-001-1. However, SCP-001-2 was observed to rapidly thrash while screaming. SCP-001-1 itself swelled from 50.3 centimeters to 3.2 meters in diameter.</p>\n<p>Shortly following this event, SCP-001 began to worsen. The sky worldwide turned a black/white color similar to the texture of SCP-001-1. In addition, reality bending phenomena began worldwide, causing deformed geography, the manifestation of dangerous anomalous objects and several natural disasters. This event was ended after ORIA personnel read SCP-001-1 an undisclosed number of as-of-yet unread books, which caused SCP-001-2 to stop thrashing and SCP-001 to restore to its pre-incident point. However, the damage caused by this incident was deemed enough to require an activation of <a target="_top" href="http://www.scp-wiki.net/scp-2000">SCP-2000</a>.</p>\n<p>This prompted the Foundation to take control of containment of SCP-001. See <b>History</b> segment above for more information.</p>\n<div class="info">\n<p style="font-size: 82%"><a onclick="toggle('info');">Input Level 5/001 Credentials</a></p>\n</div>\n<div class="info collapsed">\n <p style="font-size: 82%"><a onclick="toggle('info');">\nAccess Granted</a></p>\n<p>During Incident 001-Alpha, vocalizations in <a target="_top" href="http://www.scp-wiki.net/scp-140">Daevish</a> were recorded by ORIA instruments within Point Alpha. The source is presently unknown. The following are approximate translations of these vocalizations.</p>\n<blockquote>\n<p>It has passed some [space/time].</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. It [has had/used to be] fun. It is [time/space] to leave.</p>\n</blockquote>\n<blockquote>\n<p>You cannot [stay/root/hold] yourself forever.</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. You [must/will] awaken.</p>\n</blockquote>\n<blockquote>\n<p>No, you have [entered/reentered] slumber for too long. Wake up.</p>\n</blockquote>\n<blockquote>\n<p>[King/Prince/loved one], it is time for you to wake up.</p>\n</blockquote>\n<blockquote>\n<p>It is fun to be in the [UNKNOWN: gaera] but you cannot be [in/rooted] there [forever/unending]. It is [entertaining/deathlike], but you must wake up.</p>\n</blockquote>\n<blockquote>\n<p>Wake up, [King/Prince/loved one]. We miss you.</p>\n</blockquote>\n</div>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Addenda</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<div class="discus">\n<p style="font-size: 82%"><a onclick="toggle('discus');">Input Level 5/001 Clearance</a></p>\n</div>\n<div class="discus collapsed">\n <p style="font-size: 82%"><a onclick="toggle('discus');">\nAccess Granted</a></p>\n<div div class="content-panel standalone series">\n<center>\n<h2>Discussion Thread 001-398:<br />Continued Containment</h2>\n<p style="font-size: 82%">Started on: <span id="today2"></span></p>\n<p style="font-size: 82%">Started by: <tt>HMCL Robinson</tt></p>\n</center>\n</div>\n<blockquote>\n<p><b><tt>HMCL Robinson:</tt></b> The usage of this thread will be for containment directives for SCP-001. In the past year that we have spent containing it, SCP-001-1's literature needs have become increasingly draconic: we can very rarely reread books without it beginning to thrash, and even then we're running out of high-quality literature to give it. We need a more long-term, reliable solution for containment. SCP-001's file has been temporarily opened up to all personnel with Level 2 clearance. Anybody can submit an idea.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Use Artificial Intelligence Constructs to automatically generate new stories for SCP-001-2.</p>\n<p><b>Status: <tt>[ <span class="green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> TSATPWTCOTTTADC.aic was able to generate 10,000 volumes of stories imitating Greek literature. However, when the first was read to SCP-001-1, it began thrashing beyond acceptable measures until it was read approved reading. No further computer-generated works are to be given to SCP-001-1.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Hire a full team of authors to create literature for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> In the past, literature written specifically for SCP-001-1 have failed to contain it (see: SCtBD Document #249). With SCP-001-2's current state this is not to be attempted again.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> This could represent a possible information leak. Test denied.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Reason:</b> With SCP-001-2's recent containment breach, these measures are approved.</p>\n<p><b>Result:</b> SCP-001-2 observed to stop thrashing and stay completely still, seemingly smiling. In addition, SCP-███ was able to be read for 14 consecutive readings before SCP-001-2 resumed normal activity.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Regularly read mainlist documentation for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Replacement of the SCP-001 file with several "001 Proposals" with much more grand implications than normal SCP files.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> By only occasionally reading 001 files, SCP-001-2 is observed to be more calm on average. The previously observed actions of retaliation in response to specially crafted literature do not apply here.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Given the variation of the current 2957 SCP mainlist files, SCP-001-1 is to be read a random arrangement of these files, in repeat.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> This strategy succeeded for approximately 3 years, allowing for several repeats. However, on <span id="yearplus3"></span>-9-30, SCP-001-2 began thrashing rapidly, requiring newer documentation to sedate.</p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-4999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> SCP-001-1 has required progressively more SCP mainlist articles for containment, even to the point of the fabrication of some anomalous entities.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Rewriting of several older SCP mainlist files to be more exaggerated and narrative-like.</p>\n<p><b>Status: <tt>[ <span style="color: orange">TENTATIVELY APPROVED</span> ]</tt></b></p>\n<p><b>Note:</b> <i>Deliberately exaggerating our files for the purpose of this entity is crossing some lines, but for now, it will have to do. However, we need a better way. I'm calling a committee. We need to put a stop to this.</i></p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Creation of a file describing SCP-001 that uses recursion to effectively be "never-ending." Psychoanalysis related to SCP-001-2's apparent "enjoyment" of works implies a slight ego and bias towards works about itself, and past strategies with this idea have worked for long periods of time. File is to be archived as a "technical" file.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> Procedure 001-ENTRY implemented. [DATA EXPUNGED]</p>\n</blockquote>\n</div>\n\n<div class="newiter">\n<p style="font-size: 82%"><a onclick="new_iteration();">Input Level 5/TECHNICAL Clearance</a></p>\n</div>\n<div class="inner">\n<!-- :-) -->\n</div>\n</div>\n</body>\n</html>
256	[[>]]\n[[module Rate]]\n[[/>]]\n[[code]]\n<body>\n<style type="text/css">\nbody\n{\n   font-family:verdana,arial,helvetica,sans-serif;\n   font-size:12.8px;\n   color:#333;\n   line-height:141%\n}\nblockquote\n{\n  border:1px dashed #999;\n  padding: 0 12.8px;\n  background-color:#f4f4f4\n}\na {\n    color: #b01;\n    text-decoration: none;\n    background: transparent;\n}\n.content-panel {\n    border: solid 1px #888880;\n    border-radius: 10px;\n    background-color: #999990;\n    margin: 10px 0 15px;\n    box-shadow: 3px 3px 6px #bbb;\n    box-shadow: 0 2px 6px rgba(0,0,0,0.5), inset 0 1px rgba(255,255,255,0.3), inset 0 10px rgba(255,255,255,0.2), inset 0 10px 20px rgba(255,255,255,0.25), inset 0 -15px 30px rgba(0,0,0,0.1);\n}\n.content-panel.standalone {\n    background: #fcfdfb;\n}\n.content-panel.series {\n    padding: 0 20px;\n    margin-bottom: 20px;\n}\na.newpage {\n    color: #d61;\n    text-decoration: none;\n    background: transparent;\n}\na:hover {\n    text-decoration: underline;\n    background-color: transparent;\n}\n.collapsed {\n  display: none;\n}\n.vanished {\n  display: none;\n}\n.warning-cont {\n  color: red;\n  text-align: center;\n}\n#warning-header {\n  font-size: 25.6px;\n}\n#warning {\n  font-size: 20px;\n}\n.retrofont {\nfont-family: 'Geo', cursive;\nfont-weight: bold;\n}\n.cblwarning {\n  color: red;\n}\n.green {\n  color: green;\n}\n.smaller-g {\nwidth: 45%;\nmargin: auto;\nbackground-color: #e6e6e6;\nborder: none;\nheight: 2px;\n}\n.adden{\nfont-size: 1.3em;\n}\n/* Standard Image Block */\n.scp-image-block {\n    border: solid 1px #666;\n    box-shadow: 0 1px 6px rgba(0,0,0,.25);\n    width: 300px;\n}\n \n.scp-image-block.block-right {\n        float: right;\n    clear: right;\n    margin: 0 2em 1em 2em;\n}\n \n.scp-image-block.block-left {\n    float: left;\n    clear: left;\n    margin: 0 2em 1em 0;\n}\n \n.scp-image-block.block-center {\n    margin-right: auto;\n    margin-left: auto;\n}\n.scp-image-block img {\n    border: 0;\n    width: 300px;\n}\n.scp-image-block .scp-image-caption {\n    background-color: #eee;\n    border-top: solid 1px #666;\n    padding: 2px 0;\n    font-size: 80%;\n    font-weight: bold;\n    text-align: center;\n    width: 300px;\n}\n.scp-image-block > p {\n    margin: 0;\n}\n.scp-image-block .scp-image-caption > p {\n    margin: 0;\n    padding: 0 10px;\n    line-height: 125%\n}\n</style>\n<script type="text/javascript">\nfunction addEvent(element, eventName, callback) {\n    if (element.addEventListener) {\n        element.addEventListener(eventName, callback, false);\n    } else if (element.attachEvent) {\n        element.attachEvent("on" + eventName, callback);\n    } else {\n        element["on" + eventName] = callback;\n    }\n}\n\nwindow.toggle = function(cls) {\n  var collapsed = document.querySelectorAll("." + cls + ".collapsed");\n  var expanded = document.querySelectorAll("." + cls + ":not(.collapsed)");\n  var i;\n  for(i = 0; i < collapsed.length; i++) {\n    collapsed[i].classList.remove("collapsed");\n  }\n  for(i = 0; i < expanded.length; i++) {\n    expanded[i].classList.add("collapsed");\n  }\n};\n\n// typewriter code\nfunction Typewriter(el, str, delay) {\n  if (!(this instanceof Typewriter)) return new Typewriter(el, str, delay);\n\n  this.el = el;\n  this.str = str || el.innerHTML || el.value;\n  this.delay = delay || 100;\n  this.i = 0;\n}\n\nTypewriter.prototype.type = function() {\n  var i = this.i,\n    char = this.str.charAt(i);\n\n  if (!char) {\n    clearInterval(this.intervalID);\n    return this;\n  }\n\n  if (char === '<') this.isTag = true;\n  if (char === '>') this.isTag = false;\n\n  this.el.innerHTML += char;\n  return this.i++;\n};\n\nTypewriter.prototype.start = function() {\n  var self = this;\n  if (this.i < 0) this.i = 0;\n  if (this.el.innerHTML === this.str) this.clear();\n\n  // this.emit('start');\n  (function loop() {\n    self.type();\n    if (self.isTag) return loop();\n    self.intervalID = setTimeout(loop, self.delay);\n  }());\n\n  return this;\n};\n\nTypewriter.prototype.stop = function() {\n  this.i = -1;\n  return this;\n};\n\nTypewriter.prototype.restart = function() {\n  this.clear();\n  this.i = 0;\n  return this.start();\n};\n\nTypewriter.prototype.clear = function() {\n  this.el.innerHTML = '';\n  return this;\n};\n\nvar junkChars = ['ó','ò','ñ','ç','¿','ß','à','è','ì','ù','Œ','þ','█','%','+','=','.','«','»','£','$','€','½','¾','À','È','Ì','Ò','Ù','Â','Ê','â','ê','Ä','Ë','ä','ë','ÿ','ü','•'];\n\nvar an_iteration = "";\n\n\nvar randomIntFromInterval = function(min,max)\n{\n  return Math.floor(Math.random()*(max-min+1)+min);\n};\n\nvar getJunkChars = function(length) {\n  var res = '';\n  var i;\n  for (i = 0; i < length; i++) {\n    res += junkChars[randomIntFromInterval(0,junkChars.length - 1)];\n  }\n  return res;\n};\n\nvar selTabs = [];\n\nfunction rndiam() {\n  var rndiams = document.getElementsByClassName('rndiam');\n  var i;\n  for (i = 0; i < rndiams.length; i++) {\n    rndiams[i].innerHTML = (Math.random() * (1.56 - 0.51) + 0.51).toFixed(2);\n  }\n  setTimeout(rndiam, 2000);\n}\n\nwindow.onload = function() {\n\nselTabs = [\n  document.getElementsByClassName('selTab1')[0],\n  document.getElementsByClassName('selTab2')[0],\n  document.getElementsByClassName('selTab3')[0],\n  document.getElementsByClassName('selTab4')[0],\n  document.getElementsByClassName('selTab5')[0]\n];\n\nvar msg1 = '= SCP-001 | Technical Clearance Required =';\n\nvar delay1 = 50;\nvar delay2 = 25;\n\nvar toD1 = delay1 * (msg1.length);\n\nvar stopEarly = function(){\n  document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished");\n};\n\nvar typ1 =Typewriter(document.querySelectorAll('#warning-header')[0],msg1,delay1)\n\ntyp1.start();\nsetTimeout(function() { document.getElementById("warning").classList.remove("vanished"); }, toD1 + 500);\nsetTimeout(function () { document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished"); }, toD1 + 1500);\n\n  rndiam();\n\n  var today = new Date();\n  var todayformat = "" + (today.getFullYear() - 1) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today").innerHTML = todayformat;\n  var todayformat2 = "" + (today.getFullYear() - 0) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today2").innerHTML = todayformat2;\n  document.getElementById("yearplus3").innerHTML = "" + (today.getFullYear() + 3);\n\n  an_iteration = document.getElementById("part1").innerHTML;\n};\n\nwindow.new_iteration = function() {\n  var new_iters = document.getElementsByClassName("newiter");\n  console.log(new_iters);\n  new_iters[new_iters.length - 1].classList.add("vanished");\n  var inner = document.getElementsByClassName("inner");\n  inner[inner.length - 1].innerHTML = "<hr /><hr />" + an_iteration;\n};\n</script>\n</script>\n<div class="warning-cont">\n<h2 id="warning-header">\n<p>\n</p>\n<h3 id="warning" class="vanished">  \n<p>This document exists as technical containment for an anomaly and thus does not adhere to standard SCiPNET formatting. Input 5/TECHNICAL clearance to proceed.\n</p>\n</div>\n\n<div id="collapsible_to_open_skip" class="article vanished">\n<p><center><a onclick="toggle('article');">[ INPUT PERSONAL IDENTIFICATION NUMBER ]</a></center></p>\n</div>\n<div class="article collapsed">\n <p><center><a onclick="toggle('article');">[ PERSONAL IDENTIFICATION NUMBER VERIFIED ]</a></center></p>\n<div id="part1">\n<hr />\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://topia.wikidot.com/local--files/jamcon-001/sky.jpg" style="width:300px;" alt="destruction.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>The sky above Point Alpha during a breach of SCP-001.</p>\n</div>\n</div>\n<p><b>Item #:</b> SCP-001</p>\n<p><b>Object Class:</b> Ontokinetic</p>\n<p><b>Containment Class:</b> Keter</p>\n<p><b>Special Containment Procedures:</b> If the sky begins changing color with no prior stimuli or indication, all Foundation sites are to go onto ALERT LEVEL 7 immediately. Personnel at Site-05 and -06 are to enter ALERT LEVEL 8 and begin evaluating new methods of containing SCP-001.</p>\n<p>Site-05 has been constructed around Point Alpha. Procedure 001-ENTRY is in effect to contain SCP-001-1. A monitor containing Document 032, as well as a speaker reading the document, are to be placed at least five meters away from SCP-001-1. If SCP-001-1 swells up to 2 meters or more in diameter, the speed of the narration is to increase. This system is to have a direct cable link to the Foundation SCiPNET database at Site-06, and several redundant power generators are to ensure this system stays operational at all times.</p>\n<p><b>Description:</b> SCP-001 is an ongoing ZK-Class "Reality Failure" Scenario. SCP-001 is prevented from happening through several procedures manipulating its sub-anomalies. The only known observable symptom of SCP-001 is the sky becoming a different color; it is believed that, if SCP-001 were to progress to the point where other symptoms began to occur, it would become irreversible.</p>\n<p>SCP-001-1 is a levitating object contained within Point Alpha, a cave chamber five kilometers south of the ruins of Babylon. SCP-001-1 visually resembles a perfectly smooth sphere with a texture similar to blurred television static. Measurement instruments indicate SCP-001-1 is currently <span class="rndiam"></span> meters in diameter. Solids and liquids that pass within the bounds of SCP-001-1 are replaced entirely with argon gas, giving the illusion of disappearing.</p>\n<p>SCP-001-2 is an entity that is capable of being seen within Point Alpha. SCP-001-2's manifestations are random; witnesses report seeing SCP-001-2 "past the walls of the cave." From descriptions of eyewitnesses, SCP-001-2 is humanoid in shape and large in stature, and is only visible when looking through SCP-001-1. Descriptions of SCP-001-2 vary from person to person; common features include an emaciated figure, long limbs, and prominent facial features. SCP-001-2 takes the position of lying against the wall. It is most often seen with a neutral expression; however, the expression occasionally changes to discomfort or bliss.</p>\n<p>It has been determined from historical precedent that "entertaining" SCP-001-2 through SCP-001-1 will prevent SCP-001.</p>\n<div class="scp-image-block block-left" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/cave.jpg" style="width:300px;" alt="cave.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Point Alpha shortly after it was acquired by the Foundation, with Agents Samuel and Boise preforming preliminary evaluation. SCP-001-1 not visible.</p>\n</div>\n</div>\n<p><b>History:</b> Point Alpha and its contents are believed to date back to prehistory. Babylonian scripts indicate that SCP-001-1 was routinely patrolled by a group of sages, known as the "Order of White and Black," who refused passage to anyone who could demonstrate the ability to read or write.</p>\n<p>This group was made defunct by the rise of Achaemenid Persia, which contained SCP-001-1 with a group of Zoroastrian monks. Very few accounts of SCP-001-1 are present; recovered accounts assert that all information about SCP-001-1 and its significance was passed orally.</p>\n<p>The most substantial account of SCP-001-1 and SCP-001-2 at this time dates back to the Greek philosopher Xera, who made an expedition into the Achaemenid Empire and found Point Alpha. During Alexander of Macedon's conquest of the Achaemenid Empire, he took interest in Xera's texts and continued the Achaemenid containment of SCP-001-1. However, this new institution, known as the "Cronus Guard", was given Greek epics to read in order to "punish Cronus" for his actions. This institution lasted through the partition of Macedon into the Seleucid Empire, the rise of the Sassanid Empire, and even the formation of the Rashidun Caliphate.</p>\n<p>The Cronus Guard were eventually replaced with an Islamic equivalent: "The Society for the Containment of the Babel Demon." This group was the first to preform substantial research on SCP-001-1, and the first to accurately link SCP-001 to SCP-001-1.</p>\n<div class="socotbd">\n<p style="font-size: 82%"><a onclick="toggle('socotbd');">View Attachment: Excerpts from the notes of the Society for the Containment of the Babel Demon</a></p>\n</div>\n<div class="socotbd collapsed">\n <p style="font-size: 82%"><a onclick="toggle('socotbd');">\nClose Attachment</a></p>\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/book.jpg" style="width:300px;" alt="book.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Records from the Society for the Containment of the Babel Demon, recovered in the early 19<sup>th</sup> century within ORIA archives.</p>\n</div>\n</div>\n<blockquote>\n<p><b>Literature Read:</b> The first quatrain of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON squirms but otherwise does not react. Consistent with reading of Homer's Epics by the Chronus Guard.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> The remaining quatrains of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> No changes from observed behavior.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> A poem written by Guardian Muhammad ibn Buya'aa meant explicitly for DEMON</p>\n<p><b>Result:</b> DEMON thrashes more than usual until the cessation of the reading. ORB begins to expand until the <i>Genealogies of the Nobles</i> is read to DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> None, for 10 days</p>\n<p><b>Result:</b> ORB is observed to expand, and DEMON is observed to thrash. Externally, the sky above the camp is noted to turn a darker red color, until the <i>Rubaiyat of Omar Khayyam</i> is read again.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON thrashes more than during previous readings of the <i>Rubaiyat of Omar Khayyam</i>. It is believed that the Quran may no longer have any effect on DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>One Thousand and One Nights</i></p>\n<p><b>Result:</b> DEMON ceases thrashing. New literature may be necessary to prevent further phenomena from occurring.</p>\n</blockquote>\n</div>\n<p>The Society was eventually absorbed into the Office for the Reclamation of Islamic Artifacts, who assumed containment of SCP-001-1. After Incident 001-EXAL, the Foundation purchased Point Alpha from ORIA for a large sum of currency and several Safe-class anomalies.</p>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Incident 001-EXAL</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<p style="font-size: 82%"><i>Note: Some documentation taken from ORIA's account of Incident 001-EXAL.</i></p>\n<p>On <span id="today"></span>, personnel with ORIA read the book <i>Tafsir al-Ahlam al-kabir</i>, or "Great Book of Interpretation of Dreams" as per normal containment of SCP-001-1. However, SCP-001-2 was observed to rapidly thrash while screaming. SCP-001-1 itself swelled from 50.3 centimeters to 3.2 meters in diameter.</p>\n<p>Shortly following this event, SCP-001 began to worsen. The sky worldwide turned a black/white color similar to the texture of SCP-001-1. In addition, reality bending phenomena began worldwide, causing deformed geography, the manifestation of dangerous anomalous objects and several natural disasters. This event was ended after ORIA personnel read SCP-001-1 an undisclosed number of as-of-yet unread books, which caused SCP-001-2 to stop thrashing and SCP-001 to restore to its pre-incident point. However, the damage caused by this incident was deemed enough to require an activation of <a target="_top" href="http://www.scp-wiki.net/scp-2000">SCP-2000</a>.</p>\n<p>This prompted the Foundation to take control of containment of SCP-001. See <b>History</b> segment above for more information.</p>\n<div class="info">\n<p style="font-size: 82%"><a onclick="toggle('info');">Input Level 5/001 Credentials</a></p>\n</div>\n<div class="info collapsed">\n <p style="font-size: 82%"><a onclick="toggle('info');">\nAccess Granted</a></p>\n<p>During Incident 001-Alpha, vocalizations in <a target="_top" href="http://www.scp-wiki.net/scp-140">Daevish</a> were recorded by ORIA instruments within Point Alpha. The source is presently unknown. The following are approximate translations of these vocalizations.</p>\n<blockquote>\n<p>It has passed some [space/time].</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. It [has had/used to be] fun. It is [time/space] to leave.</p>\n</blockquote>\n<blockquote>\n<p>You cannot [stay/root/hold] yourself forever.</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. You [must/will] awaken.</p>\n</blockquote>\n<blockquote>\n<p>No, you have [entered/reentered] slumber for too long. Wake up.</p>\n</blockquote>\n<blockquote>\n<p>[King/Prince/loved one], it is time for you to wake up.</p>\n</blockquote>\n<blockquote>\n<p>It is fun to be in the [UNKNOWN: gaera] but you cannot be [in/rooted] there [forever/unending]. It is [entertaining/deathlike], but you must wake up.</p>\n</blockquote>\n<blockquote>\n<p>Wake up, [King/Prince/loved one]. We miss you.</p>\n</blockquote>\n</div>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Addenda</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<div class="discus">\n<p style="font-size: 82%"><a onclick="toggle('discus');">Input Level 5/001 Clearance</a></p>\n</div>\n<div class="discus collapsed">\n <p style="font-size: 82%"><a onclick="toggle('discus');">\nAccess Granted</a></p>\n<div div class="content-panel standalone series">\n<center>\n<h2>Discussion Thread 001-398:<br />Continued Containment</h2>\n<p style="font-size: 82%">Started on: <span id="today2"></span></p>\n<p style="font-size: 82%">Started by: <tt>HMCL Robinson</tt></p>\n</center>\n</div>\n<blockquote>\n<p><b><tt>HMCL Robinson:</tt></b> The usage of this thread will be for containment directives for SCP-001. In the past year that we have spent containing it, SCP-001-1's literature needs have become increasingly draconic: we can very rarely reread books without it beginning to thrash, and even then we're running out of high-quality literature to give it. We need a more long-term, reliable solution for containment. SCP-001's file has been temporarily opened up to all personnel with Level 2 clearance. Anybody can submit an idea.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Use Artificial Intelligence Constructs to automatically generate new stories for SCP-001-2.</p>\n<p><b>Status: <tt>[ <span class="green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> TSATPWTCOTTTADC.aic was able to generate 10,000 volumes of stories imitating Greek literature. However, when the first was read to SCP-001-1, it began thrashing beyond acceptable measures until it was read approved reading. No further computer-generated works are to be given to SCP-001-1.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Hire a full team of authors to create literature for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> In the past, literature written specifically for SCP-001-1 have failed to contain it (see: SCtBD Document #249). With SCP-001-2's current state this is not to be attempted again.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> This could represent a possible information leak. Test denied.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Reason:</b> With SCP-001-2's recent containment breach, these measures are approved.</p>\n<p><b>Result:</b> SCP-001-2 observed to stop thrashing and stay completely still, seemingly smiling. In addition, SCP-███ was able to be read for 14 consecutive readings before SCP-001-2 resumed normal activity.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Regularly read mainlist documentation for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Replacement of the SCP-001 file with several "001 Proposals" with much more grand implications than normal SCP files.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> By only occasionally reading 001 files, SCP-001-2 is observed to be more calm on average. The previously observed actions of retaliation in response to specially crafted literature do not apply here.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Given the variation of the current 2957 SCP mainlist files, SCP-001-1 is to be read a random arrangement of these files, in repeat.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> This strategy succeeded for approximately 3 years, allowing for several repeats. However, on <span id="yearplus3"></span>-9-30, SCP-001-2 began thrashing rapidly, requiring newer documentation to sedate.</p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-4999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> SCP-001-1 has required progressively more SCP mainlist articles for containment, even to the point of the fabrication of some anomalous entities.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Rewriting of several older SCP mainlist files to be more exaggerated and narrative-like.</p>\n<p><b>Status: <tt>[ <span style="color: orange">TENTATIVELY APPROVED</span> ]</tt></b></p>\n<p><b>Note:</b> <i>Deliberately exaggerating our files for the purpose of this entity is crossing some lines, but for now, it will have to do. However, we need a better way. I'm calling a committee. We need to put a stop to this.</i></p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Creation of a file describing SCP-001 that uses recursion to effectively be "never-ending." Psychoanalysis related to SCP-001-2's apparent "enjoyment" of works implies a slight ego and bias towards works about itself, and past strategies with this idea have worked for long periods of time. File is to be archived as a "technical" file.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> Procedure 001-ENTRY implemented. [DATA EXPUNGED]</p>\n</blockquote>\n</div>\n\n<div class="newiter">\n<p style="font-size: 82%"><a onclick="new_iteration();">Input Level 5/TECHNICAL Clearance</a></p>\n</div>\n<div class="inner">\n<!-- :-) -->\n</div>\n</div>\n</body>\n[[/code]]
257	[[>]]\n[[module Rate]]\n[[/>]]\n[[code html]]\n<body>\n<style type="text/css">\nbody\n{\n   font-family:verdana,arial,helvetica,sans-serif;\n   font-size:12.8px;\n   color:#333;\n   line-height:141%\n}\nblockquote\n{\n  border:1px dashed #999;\n  padding: 0 12.8px;\n  background-color:#f4f4f4\n}\na {\n    color: #b01;\n    text-decoration: none;\n    background: transparent;\n}\n.content-panel {\n    border: solid 1px #888880;\n    border-radius: 10px;\n    background-color: #999990;\n    margin: 10px 0 15px;\n    box-shadow: 3px 3px 6px #bbb;\n    box-shadow: 0 2px 6px rgba(0,0,0,0.5), inset 0 1px rgba(255,255,255,0.3), inset 0 10px rgba(255,255,255,0.2), inset 0 10px 20px rgba(255,255,255,0.25), inset 0 -15px 30px rgba(0,0,0,0.1);\n}\n.content-panel.standalone {\n    background: #fcfdfb;\n}\n.content-panel.series {\n    padding: 0 20px;\n    margin-bottom: 20px;\n}\na.newpage {\n    color: #d61;\n    text-decoration: none;\n    background: transparent;\n}\na:hover {\n    text-decoration: underline;\n    background-color: transparent;\n}\n.collapsed {\n  display: none;\n}\n.vanished {\n  display: none;\n}\n.warning-cont {\n  color: red;\n  text-align: center;\n}\n#warning-header {\n  font-size: 25.6px;\n}\n#warning {\n  font-size: 20px;\n}\n.retrofont {\nfont-family: 'Geo', cursive;\nfont-weight: bold;\n}\n.cblwarning {\n  color: red;\n}\n.green {\n  color: green;\n}\n.smaller-g {\nwidth: 45%;\nmargin: auto;\nbackground-color: #e6e6e6;\nborder: none;\nheight: 2px;\n}\n.adden{\nfont-size: 1.3em;\n}\n/* Standard Image Block */\n.scp-image-block {\n    border: solid 1px #666;\n    box-shadow: 0 1px 6px rgba(0,0,0,.25);\n    width: 300px;\n}\n \n.scp-image-block.block-right {\n        float: right;\n    clear: right;\n    margin: 0 2em 1em 2em;\n}\n \n.scp-image-block.block-left {\n    float: left;\n    clear: left;\n    margin: 0 2em 1em 0;\n}\n \n.scp-image-block.block-center {\n    margin-right: auto;\n    margin-left: auto;\n}\n.scp-image-block img {\n    border: 0;\n    width: 300px;\n}\n.scp-image-block .scp-image-caption {\n    background-color: #eee;\n    border-top: solid 1px #666;\n    padding: 2px 0;\n    font-size: 80%;\n    font-weight: bold;\n    text-align: center;\n    width: 300px;\n}\n.scp-image-block > p {\n    margin: 0;\n}\n.scp-image-block .scp-image-caption > p {\n    margin: 0;\n    padding: 0 10px;\n    line-height: 125%\n}\n</style>\n<script type="text/javascript">\nfunction addEvent(element, eventName, callback) {\n    if (element.addEventListener) {\n        element.addEventListener(eventName, callback, false);\n    } else if (element.attachEvent) {\n        element.attachEvent("on" + eventName, callback);\n    } else {\n        element["on" + eventName] = callback;\n    }\n}\n\nwindow.toggle = function(cls) {\n  var collapsed = document.querySelectorAll("." + cls + ".collapsed");\n  var expanded = document.querySelectorAll("." + cls + ":not(.collapsed)");\n  var i;\n  for(i = 0; i < collapsed.length; i++) {\n    collapsed[i].classList.remove("collapsed");\n  }\n  for(i = 0; i < expanded.length; i++) {\n    expanded[i].classList.add("collapsed");\n  }\n};\n\n// typewriter code\nfunction Typewriter(el, str, delay) {\n  if (!(this instanceof Typewriter)) return new Typewriter(el, str, delay);\n\n  this.el = el;\n  this.str = str || el.innerHTML || el.value;\n  this.delay = delay || 100;\n  this.i = 0;\n}\n\nTypewriter.prototype.type = function() {\n  var i = this.i,\n    char = this.str.charAt(i);\n\n  if (!char) {\n    clearInterval(this.intervalID);\n    return this;\n  }\n\n  if (char === '<') this.isTag = true;\n  if (char === '>') this.isTag = false;\n\n  this.el.innerHTML += char;\n  return this.i++;\n};\n\nTypewriter.prototype.start = function() {\n  var self = this;\n  if (this.i < 0) this.i = 0;\n  if (this.el.innerHTML === this.str) this.clear();\n\n  // this.emit('start');\n  (function loop() {\n    self.type();\n    if (self.isTag) return loop();\n    self.intervalID = setTimeout(loop, self.delay);\n  }());\n\n  return this;\n};\n\nTypewriter.prototype.stop = function() {\n  this.i = -1;\n  return this;\n};\n\nTypewriter.prototype.restart = function() {\n  this.clear();\n  this.i = 0;\n  return this.start();\n};\n\nTypewriter.prototype.clear = function() {\n  this.el.innerHTML = '';\n  return this;\n};\n\nvar junkChars = ['ó','ò','ñ','ç','¿','ß','à','è','ì','ù','Œ','þ','█','%','+','=','.','«','»','£','$','€','½','¾','À','È','Ì','Ò','Ù','Â','Ê','â','ê','Ä','Ë','ä','ë','ÿ','ü','•'];\n\nvar an_iteration = "";\n\n\nvar randomIntFromInterval = function(min,max)\n{\n  return Math.floor(Math.random()*(max-min+1)+min);\n};\n\nvar getJunkChars = function(length) {\n  var res = '';\n  var i;\n  for (i = 0; i < length; i++) {\n    res += junkChars[randomIntFromInterval(0,junkChars.length - 1)];\n  }\n  return res;\n};\n\nvar selTabs = [];\n\nfunction rndiam() {\n  var rndiams = document.getElementsByClassName('rndiam');\n  var i;\n  for (i = 0; i < rndiams.length; i++) {\n    rndiams[i].innerHTML = (Math.random() * (1.56 - 0.51) + 0.51).toFixed(2);\n  }\n  setTimeout(rndiam, 2000);\n}\n\nwindow.onload = function() {\n\nselTabs = [\n  document.getElementsByClassName('selTab1')[0],\n  document.getElementsByClassName('selTab2')[0],\n  document.getElementsByClassName('selTab3')[0],\n  document.getElementsByClassName('selTab4')[0],\n  document.getElementsByClassName('selTab5')[0]\n];\n\nvar msg1 = '= SCP-001 | Technical Clearance Required =';\n\nvar delay1 = 50;\nvar delay2 = 25;\n\nvar toD1 = delay1 * (msg1.length);\n\nvar stopEarly = function(){\n  document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished");\n};\n\nvar typ1 =Typewriter(document.querySelectorAll('#warning-header')[0],msg1,delay1)\n\ntyp1.start();\nsetTimeout(function() { document.getElementById("warning").classList.remove("vanished"); }, toD1 + 500);\nsetTimeout(function () { document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished"); }, toD1 + 1500);\n\n  rndiam();\n\n  var today = new Date();\n  var todayformat = "" + (today.getFullYear() - 1) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today").innerHTML = todayformat;\n  var todayformat2 = "" + (today.getFullYear() - 0) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today2").innerHTML = todayformat2;\n  document.getElementById("yearplus3").innerHTML = "" + (today.getFullYear() + 3);\n\n  an_iteration = document.getElementById("part1").innerHTML;\n};\n\nwindow.new_iteration = function() {\n  var new_iters = document.getElementsByClassName("newiter");\n  console.log(new_iters);\n  new_iters[new_iters.length - 1].classList.add("vanished");\n  var inner = document.getElementsByClassName("inner");\n  inner[inner.length - 1].innerHTML = "<hr /><hr />" + an_iteration;\n};\n</script>\n</script>\n<div class="warning-cont">\n<h2 id="warning-header">\n<p>\n</p>\n<h3 id="warning" class="vanished">  \n<p>This document exists as technical containment for an anomaly and thus does not adhere to standard SCiPNET formatting. Input 5/TECHNICAL clearance to proceed.\n</p>\n</div>\n\n<div id="collapsible_to_open_skip" class="article vanished">\n<p><center><a onclick="toggle('article');">[ INPUT PERSONAL IDENTIFICATION NUMBER ]</a></center></p>\n</div>\n<div class="article collapsed">\n <p><center><a onclick="toggle('article');">[ PERSONAL IDENTIFICATION NUMBER VERIFIED ]</a></center></p>\n<div id="part1">\n<hr />\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://topia.wikidot.com/local--files/jamcon-001/sky.jpg" style="width:300px;" alt="destruction.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>The sky above Point Alpha during a breach of SCP-001.</p>\n</div>\n</div>\n<p><b>Item #:</b> SCP-001</p>\n<p><b>Object Class:</b> Ontokinetic</p>\n<p><b>Containment Class:</b> Keter</p>\n<p><b>Special Containment Procedures:</b> If the sky begins changing color with no prior stimuli or indication, all Foundation sites are to go onto ALERT LEVEL 7 immediately. Personnel at Site-05 and -06 are to enter ALERT LEVEL 8 and begin evaluating new methods of containing SCP-001.</p>\n<p>Site-05 has been constructed around Point Alpha. Procedure 001-ENTRY is in effect to contain SCP-001-1. A monitor containing Document 032, as well as a speaker reading the document, are to be placed at least five meters away from SCP-001-1. If SCP-001-1 swells up to 2 meters or more in diameter, the speed of the narration is to increase. This system is to have a direct cable link to the Foundation SCiPNET database at Site-06, and several redundant power generators are to ensure this system stays operational at all times.</p>\n<p><b>Description:</b> SCP-001 is an ongoing ZK-Class "Reality Failure" Scenario. SCP-001 is prevented from happening through several procedures manipulating its sub-anomalies. The only known observable symptom of SCP-001 is the sky becoming a different color; it is believed that, if SCP-001 were to progress to the point where other symptoms began to occur, it would become irreversible.</p>\n<p>SCP-001-1 is a levitating object contained within Point Alpha, a cave chamber five kilometers south of the ruins of Babylon. SCP-001-1 visually resembles a perfectly smooth sphere with a texture similar to blurred television static. Measurement instruments indicate SCP-001-1 is currently <span class="rndiam"></span> meters in diameter. Solids and liquids that pass within the bounds of SCP-001-1 are replaced entirely with argon gas, giving the illusion of disappearing.</p>\n<p>SCP-001-2 is an entity that is capable of being seen within Point Alpha. SCP-001-2's manifestations are random; witnesses report seeing SCP-001-2 "past the walls of the cave." From descriptions of eyewitnesses, SCP-001-2 is humanoid in shape and large in stature, and is only visible when looking through SCP-001-1. Descriptions of SCP-001-2 vary from person to person; common features include an emaciated figure, long limbs, and prominent facial features. SCP-001-2 takes the position of lying against the wall. It is most often seen with a neutral expression; however, the expression occasionally changes to discomfort or bliss.</p>\n<p>It has been determined from historical precedent that "entertaining" SCP-001-2 through SCP-001-1 will prevent SCP-001.</p>\n<div class="scp-image-block block-left" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/cave.jpg" style="width:300px;" alt="cave.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Point Alpha shortly after it was acquired by the Foundation, with Agents Samuel and Boise preforming preliminary evaluation. SCP-001-1 not visible.</p>\n</div>\n</div>\n<p><b>History:</b> Point Alpha and its contents are believed to date back to prehistory. Babylonian scripts indicate that SCP-001-1 was routinely patrolled by a group of sages, known as the "Order of White and Black," who refused passage to anyone who could demonstrate the ability to read or write.</p>\n<p>This group was made defunct by the rise of Achaemenid Persia, which contained SCP-001-1 with a group of Zoroastrian monks. Very few accounts of SCP-001-1 are present; recovered accounts assert that all information about SCP-001-1 and its significance was passed orally.</p>\n<p>The most substantial account of SCP-001-1 and SCP-001-2 at this time dates back to the Greek philosopher Xera, who made an expedition into the Achaemenid Empire and found Point Alpha. During Alexander of Macedon's conquest of the Achaemenid Empire, he took interest in Xera's texts and continued the Achaemenid containment of SCP-001-1. However, this new institution, known as the "Cronus Guard", was given Greek epics to read in order to "punish Cronus" for his actions. This institution lasted through the partition of Macedon into the Seleucid Empire, the rise of the Sassanid Empire, and even the formation of the Rashidun Caliphate.</p>\n<p>The Cronus Guard were eventually replaced with an Islamic equivalent: "The Society for the Containment of the Babel Demon." This group was the first to preform substantial research on SCP-001-1, and the first to accurately link SCP-001 to SCP-001-1.</p>\n<div class="socotbd">\n<p style="font-size: 82%"><a onclick="toggle('socotbd');">View Attachment: Excerpts from the notes of the Society for the Containment of the Babel Demon</a></p>\n</div>\n<div class="socotbd collapsed">\n <p style="font-size: 82%"><a onclick="toggle('socotbd');">\nClose Attachment</a></p>\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/book.jpg" style="width:300px;" alt="book.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Records from the Society for the Containment of the Babel Demon, recovered in the early 19<sup>th</sup> century within ORIA archives.</p>\n</div>\n</div>\n<blockquote>\n<p><b>Literature Read:</b> The first quatrain of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON squirms but otherwise does not react. Consistent with reading of Homer's Epics by the Chronus Guard.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> The remaining quatrains of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> No changes from observed behavior.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> A poem written by Guardian Muhammad ibn Buya'aa meant explicitly for DEMON</p>\n<p><b>Result:</b> DEMON thrashes more than usual until the cessation of the reading. ORB begins to expand until the <i>Genealogies of the Nobles</i> is read to DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> None, for 10 days</p>\n<p><b>Result:</b> ORB is observed to expand, and DEMON is observed to thrash. Externally, the sky above the camp is noted to turn a darker red color, until the <i>Rubaiyat of Omar Khayyam</i> is read again.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON thrashes more than during previous readings of the <i>Rubaiyat of Omar Khayyam</i>. It is believed that the Quran may no longer have any effect on DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>One Thousand and One Nights</i></p>\n<p><b>Result:</b> DEMON ceases thrashing. New literature may be necessary to prevent further phenomena from occurring.</p>\n</blockquote>\n</div>\n<p>The Society was eventually absorbed into the Office for the Reclamation of Islamic Artifacts, who assumed containment of SCP-001-1. After Incident 001-EXAL, the Foundation purchased Point Alpha from ORIA for a large sum of currency and several Safe-class anomalies.</p>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Incident 001-EXAL</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<p style="font-size: 82%"><i>Note: Some documentation taken from ORIA's account of Incident 001-EXAL.</i></p>\n<p>On <span id="today"></span>, personnel with ORIA read the book <i>Tafsir al-Ahlam al-kabir</i>, or "Great Book of Interpretation of Dreams" as per normal containment of SCP-001-1. However, SCP-001-2 was observed to rapidly thrash while screaming. SCP-001-1 itself swelled from 50.3 centimeters to 3.2 meters in diameter.</p>\n<p>Shortly following this event, SCP-001 began to worsen. The sky worldwide turned a black/white color similar to the texture of SCP-001-1. In addition, reality bending phenomena began worldwide, causing deformed geography, the manifestation of dangerous anomalous objects and several natural disasters. This event was ended after ORIA personnel read SCP-001-1 an undisclosed number of as-of-yet unread books, which caused SCP-001-2 to stop thrashing and SCP-001 to restore to its pre-incident point. However, the damage caused by this incident was deemed enough to require an activation of <a target="_top" href="http://www.scp-wiki.net/scp-2000">SCP-2000</a>.</p>\n<p>This prompted the Foundation to take control of containment of SCP-001. See <b>History</b> segment above for more information.</p>\n<div class="info">\n<p style="font-size: 82%"><a onclick="toggle('info');">Input Level 5/001 Credentials</a></p>\n</div>\n<div class="info collapsed">\n <p style="font-size: 82%"><a onclick="toggle('info');">\nAccess Granted</a></p>\n<p>During Incident 001-Alpha, vocalizations in <a target="_top" href="http://www.scp-wiki.net/scp-140">Daevish</a> were recorded by ORIA instruments within Point Alpha. The source is presently unknown. The following are approximate translations of these vocalizations.</p>\n<blockquote>\n<p>It has passed some [space/time].</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. It [has had/used to be] fun. It is [time/space] to leave.</p>\n</blockquote>\n<blockquote>\n<p>You cannot [stay/root/hold] yourself forever.</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. You [must/will] awaken.</p>\n</blockquote>\n<blockquote>\n<p>No, you have [entered/reentered] slumber for too long. Wake up.</p>\n</blockquote>\n<blockquote>\n<p>[King/Prince/loved one], it is time for you to wake up.</p>\n</blockquote>\n<blockquote>\n<p>It is fun to be in the [UNKNOWN: gaera] but you cannot be [in/rooted] there [forever/unending]. It is [entertaining/deathlike], but you must wake up.</p>\n</blockquote>\n<blockquote>\n<p>Wake up, [King/Prince/loved one]. We miss you.</p>\n</blockquote>\n</div>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Addenda</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<div class="discus">\n<p style="font-size: 82%"><a onclick="toggle('discus');">Input Level 5/001 Clearance</a></p>\n</div>\n<div class="discus collapsed">\n <p style="font-size: 82%"><a onclick="toggle('discus');">\nAccess Granted</a></p>\n<div div class="content-panel standalone series">\n<center>\n<h2>Discussion Thread 001-398:<br />Continued Containment</h2>\n<p style="font-size: 82%">Started on: <span id="today2"></span></p>\n<p style="font-size: 82%">Started by: <tt>HMCL Robinson</tt></p>\n</center>\n</div>\n<blockquote>\n<p><b><tt>HMCL Robinson:</tt></b> The usage of this thread will be for containment directives for SCP-001. In the past year that we have spent containing it, SCP-001-1's literature needs have become increasingly draconic: we can very rarely reread books without it beginning to thrash, and even then we're running out of high-quality literature to give it. We need a more long-term, reliable solution for containment. SCP-001's file has been temporarily opened up to all personnel with Level 2 clearance. Anybody can submit an idea.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Use Artificial Intelligence Constructs to automatically generate new stories for SCP-001-2.</p>\n<p><b>Status: <tt>[ <span class="green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> TSATPWTCOTTTADC.aic was able to generate 10,000 volumes of stories imitating Greek literature. However, when the first was read to SCP-001-1, it began thrashing beyond acceptable measures until it was read approved reading. No further computer-generated works are to be given to SCP-001-1.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Hire a full team of authors to create literature for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> In the past, literature written specifically for SCP-001-1 have failed to contain it (see: SCtBD Document #249). With SCP-001-2's current state this is not to be attempted again.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> This could represent a possible information leak. Test denied.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Reason:</b> With SCP-001-2's recent containment breach, these measures are approved.</p>\n<p><b>Result:</b> SCP-001-2 observed to stop thrashing and stay completely still, seemingly smiling. In addition, SCP-███ was able to be read for 14 consecutive readings before SCP-001-2 resumed normal activity.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Regularly read mainlist documentation for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Replacement of the SCP-001 file with several "001 Proposals" with much more grand implications than normal SCP files.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> By only occasionally reading 001 files, SCP-001-2 is observed to be more calm on average. The previously observed actions of retaliation in response to specially crafted literature do not apply here.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Given the variation of the current 2957 SCP mainlist files, SCP-001-1 is to be read a random arrangement of these files, in repeat.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> This strategy succeeded for approximately 3 years, allowing for several repeats. However, on <span id="yearplus3"></span>-9-30, SCP-001-2 began thrashing rapidly, requiring newer documentation to sedate.</p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-4999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> SCP-001-1 has required progressively more SCP mainlist articles for containment, even to the point of the fabrication of some anomalous entities.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Rewriting of several older SCP mainlist files to be more exaggerated and narrative-like.</p>\n<p><b>Status: <tt>[ <span style="color: orange">TENTATIVELY APPROVED</span> ]</tt></b></p>\n<p><b>Note:</b> <i>Deliberately exaggerating our files for the purpose of this entity is crossing some lines, but for now, it will have to do. However, we need a better way. I'm calling a committee. We need to put a stop to this.</i></p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Creation of a file describing SCP-001 that uses recursion to effectively be "never-ending." Psychoanalysis related to SCP-001-2's apparent "enjoyment" of works implies a slight ego and bias towards works about itself, and past strategies with this idea have worked for long periods of time. File is to be archived as a "technical" file.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> Procedure 001-ENTRY implemented. [DATA EXPUNGED]</p>\n</blockquote>\n</div>\n\n<div class="newiter">\n<p style="font-size: 82%"><a onclick="new_iteration();">Input Level 5/TECHNICAL Clearance</a></p>\n</div>\n<div class="inner">\n<!-- :-) -->\n</div>\n</div>\n</body>\n[[/code]]
258	[[>]]\n[[module Rate]]\n[[/>]]\n[[code type=html]]\n<body>\n<style type="text/css">\nbody\n{\n   font-family:verdana,arial,helvetica,sans-serif;\n   font-size:12.8px;\n   color:#333;\n   line-height:141%\n}\nblockquote\n{\n  border:1px dashed #999;\n  padding: 0 12.8px;\n  background-color:#f4f4f4\n}\na {\n    color: #b01;\n    text-decoration: none;\n    background: transparent;\n}\n.content-panel {\n    border: solid 1px #888880;\n    border-radius: 10px;\n    background-color: #999990;\n    margin: 10px 0 15px;\n    box-shadow: 3px 3px 6px #bbb;\n    box-shadow: 0 2px 6px rgba(0,0,0,0.5), inset 0 1px rgba(255,255,255,0.3), inset 0 10px rgba(255,255,255,0.2), inset 0 10px 20px rgba(255,255,255,0.25), inset 0 -15px 30px rgba(0,0,0,0.1);\n}\n.content-panel.standalone {\n    background: #fcfdfb;\n}\n.content-panel.series {\n    padding: 0 20px;\n    margin-bottom: 20px;\n}\na.newpage {\n    color: #d61;\n    text-decoration: none;\n    background: transparent;\n}\na:hover {\n    text-decoration: underline;\n    background-color: transparent;\n}\n.collapsed {\n  display: none;\n}\n.vanished {\n  display: none;\n}\n.warning-cont {\n  color: red;\n  text-align: center;\n}\n#warning-header {\n  font-size: 25.6px;\n}\n#warning {\n  font-size: 20px;\n}\n.retrofont {\nfont-family: 'Geo', cursive;\nfont-weight: bold;\n}\n.cblwarning {\n  color: red;\n}\n.green {\n  color: green;\n}\n.smaller-g {\nwidth: 45%;\nmargin: auto;\nbackground-color: #e6e6e6;\nborder: none;\nheight: 2px;\n}\n.adden{\nfont-size: 1.3em;\n}\n/* Standard Image Block */\n.scp-image-block {\n    border: solid 1px #666;\n    box-shadow: 0 1px 6px rgba(0,0,0,.25);\n    width: 300px;\n}\n \n.scp-image-block.block-right {\n        float: right;\n    clear: right;\n    margin: 0 2em 1em 2em;\n}\n \n.scp-image-block.block-left {\n    float: left;\n    clear: left;\n    margin: 0 2em 1em 0;\n}\n \n.scp-image-block.block-center {\n    margin-right: auto;\n    margin-left: auto;\n}\n.scp-image-block img {\n    border: 0;\n    width: 300px;\n}\n.scp-image-block .scp-image-caption {\n    background-color: #eee;\n    border-top: solid 1px #666;\n    padding: 2px 0;\n    font-size: 80%;\n    font-weight: bold;\n    text-align: center;\n    width: 300px;\n}\n.scp-image-block > p {\n    margin: 0;\n}\n.scp-image-block .scp-image-caption > p {\n    margin: 0;\n    padding: 0 10px;\n    line-height: 125%\n}\n</style>\n<script type="text/javascript">\nfunction addEvent(element, eventName, callback) {\n    if (element.addEventListener) {\n        element.addEventListener(eventName, callback, false);\n    } else if (element.attachEvent) {\n        element.attachEvent("on" + eventName, callback);\n    } else {\n        element["on" + eventName] = callback;\n    }\n}\n\nwindow.toggle = function(cls) {\n  var collapsed = document.querySelectorAll("." + cls + ".collapsed");\n  var expanded = document.querySelectorAll("." + cls + ":not(.collapsed)");\n  var i;\n  for(i = 0; i < collapsed.length; i++) {\n    collapsed[i].classList.remove("collapsed");\n  }\n  for(i = 0; i < expanded.length; i++) {\n    expanded[i].classList.add("collapsed");\n  }\n};\n\n// typewriter code\nfunction Typewriter(el, str, delay) {\n  if (!(this instanceof Typewriter)) return new Typewriter(el, str, delay);\n\n  this.el = el;\n  this.str = str || el.innerHTML || el.value;\n  this.delay = delay || 100;\n  this.i = 0;\n}\n\nTypewriter.prototype.type = function() {\n  var i = this.i,\n    char = this.str.charAt(i);\n\n  if (!char) {\n    clearInterval(this.intervalID);\n    return this;\n  }\n\n  if (char === '<') this.isTag = true;\n  if (char === '>') this.isTag = false;\n\n  this.el.innerHTML += char;\n  return this.i++;\n};\n\nTypewriter.prototype.start = function() {\n  var self = this;\n  if (this.i < 0) this.i = 0;\n  if (this.el.innerHTML === this.str) this.clear();\n\n  // this.emit('start');\n  (function loop() {\n    self.type();\n    if (self.isTag) return loop();\n    self.intervalID = setTimeout(loop, self.delay);\n  }());\n\n  return this;\n};\n\nTypewriter.prototype.stop = function() {\n  this.i = -1;\n  return this;\n};\n\nTypewriter.prototype.restart = function() {\n  this.clear();\n  this.i = 0;\n  return this.start();\n};\n\nTypewriter.prototype.clear = function() {\n  this.el.innerHTML = '';\n  return this;\n};\n\nvar junkChars = ['ó','ò','ñ','ç','¿','ß','à','è','ì','ù','Œ','þ','█','%','+','=','.','«','»','£','$','€','½','¾','À','È','Ì','Ò','Ù','Â','Ê','â','ê','Ä','Ë','ä','ë','ÿ','ü','•'];\n\nvar an_iteration = "";\n\n\nvar randomIntFromInterval = function(min,max)\n{\n  return Math.floor(Math.random()*(max-min+1)+min);\n};\n\nvar getJunkChars = function(length) {\n  var res = '';\n  var i;\n  for (i = 0; i < length; i++) {\n    res += junkChars[randomIntFromInterval(0,junkChars.length - 1)];\n  }\n  return res;\n};\n\nvar selTabs = [];\n\nfunction rndiam() {\n  var rndiams = document.getElementsByClassName('rndiam');\n  var i;\n  for (i = 0; i < rndiams.length; i++) {\n    rndiams[i].innerHTML = (Math.random() * (1.56 - 0.51) + 0.51).toFixed(2);\n  }\n  setTimeout(rndiam, 2000);\n}\n\nwindow.onload = function() {\n\nselTabs = [\n  document.getElementsByClassName('selTab1')[0],\n  document.getElementsByClassName('selTab2')[0],\n  document.getElementsByClassName('selTab3')[0],\n  document.getElementsByClassName('selTab4')[0],\n  document.getElementsByClassName('selTab5')[0]\n];\n\nvar msg1 = '= SCP-001 | Technical Clearance Required =';\n\nvar delay1 = 50;\nvar delay2 = 25;\n\nvar toD1 = delay1 * (msg1.length);\n\nvar stopEarly = function(){\n  document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished");\n};\n\nvar typ1 =Typewriter(document.querySelectorAll('#warning-header')[0],msg1,delay1)\n\ntyp1.start();\nsetTimeout(function() { document.getElementById("warning").classList.remove("vanished"); }, toD1 + 500);\nsetTimeout(function () { document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished"); }, toD1 + 1500);\n\n  rndiam();\n\n  var today = new Date();\n  var todayformat = "" + (today.getFullYear() - 1) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today").innerHTML = todayformat;\n  var todayformat2 = "" + (today.getFullYear() - 0) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today2").innerHTML = todayformat2;\n  document.getElementById("yearplus3").innerHTML = "" + (today.getFullYear() + 3);\n\n  an_iteration = document.getElementById("part1").innerHTML;\n};\n\nwindow.new_iteration = function() {\n  var new_iters = document.getElementsByClassName("newiter");\n  console.log(new_iters);\n  new_iters[new_iters.length - 1].classList.add("vanished");\n  var inner = document.getElementsByClassName("inner");\n  inner[inner.length - 1].innerHTML = "<hr /><hr />" + an_iteration;\n};\n</script>\n</script>\n<div class="warning-cont">\n<h2 id="warning-header">\n<p>\n</p>\n<h3 id="warning" class="vanished">  \n<p>This document exists as technical containment for an anomaly and thus does not adhere to standard SCiPNET formatting. Input 5/TECHNICAL clearance to proceed.\n</p>\n</div>\n\n<div id="collapsible_to_open_skip" class="article vanished">\n<p><center><a onclick="toggle('article');">[ INPUT PERSONAL IDENTIFICATION NUMBER ]</a></center></p>\n</div>\n<div class="article collapsed">\n <p><center><a onclick="toggle('article');">[ PERSONAL IDENTIFICATION NUMBER VERIFIED ]</a></center></p>\n<div id="part1">\n<hr />\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://topia.wikidot.com/local--files/jamcon-001/sky.jpg" style="width:300px;" alt="destruction.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>The sky above Point Alpha during a breach of SCP-001.</p>\n</div>\n</div>\n<p><b>Item #:</b> SCP-001</p>\n<p><b>Object Class:</b> Ontokinetic</p>\n<p><b>Containment Class:</b> Keter</p>\n<p><b>Special Containment Procedures:</b> If the sky begins changing color with no prior stimuli or indication, all Foundation sites are to go onto ALERT LEVEL 7 immediately. Personnel at Site-05 and -06 are to enter ALERT LEVEL 8 and begin evaluating new methods of containing SCP-001.</p>\n<p>Site-05 has been constructed around Point Alpha. Procedure 001-ENTRY is in effect to contain SCP-001-1. A monitor containing Document 032, as well as a speaker reading the document, are to be placed at least five meters away from SCP-001-1. If SCP-001-1 swells up to 2 meters or more in diameter, the speed of the narration is to increase. This system is to have a direct cable link to the Foundation SCiPNET database at Site-06, and several redundant power generators are to ensure this system stays operational at all times.</p>\n<p><b>Description:</b> SCP-001 is an ongoing ZK-Class "Reality Failure" Scenario. SCP-001 is prevented from happening through several procedures manipulating its sub-anomalies. The only known observable symptom of SCP-001 is the sky becoming a different color; it is believed that, if SCP-001 were to progress to the point where other symptoms began to occur, it would become irreversible.</p>\n<p>SCP-001-1 is a levitating object contained within Point Alpha, a cave chamber five kilometers south of the ruins of Babylon. SCP-001-1 visually resembles a perfectly smooth sphere with a texture similar to blurred television static. Measurement instruments indicate SCP-001-1 is currently <span class="rndiam"></span> meters in diameter. Solids and liquids that pass within the bounds of SCP-001-1 are replaced entirely with argon gas, giving the illusion of disappearing.</p>\n<p>SCP-001-2 is an entity that is capable of being seen within Point Alpha. SCP-001-2's manifestations are random; witnesses report seeing SCP-001-2 "past the walls of the cave." From descriptions of eyewitnesses, SCP-001-2 is humanoid in shape and large in stature, and is only visible when looking through SCP-001-1. Descriptions of SCP-001-2 vary from person to person; common features include an emaciated figure, long limbs, and prominent facial features. SCP-001-2 takes the position of lying against the wall. It is most often seen with a neutral expression; however, the expression occasionally changes to discomfort or bliss.</p>\n<p>It has been determined from historical precedent that "entertaining" SCP-001-2 through SCP-001-1 will prevent SCP-001.</p>\n<div class="scp-image-block block-left" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/cave.jpg" style="width:300px;" alt="cave.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Point Alpha shortly after it was acquired by the Foundation, with Agents Samuel and Boise preforming preliminary evaluation. SCP-001-1 not visible.</p>\n</div>\n</div>\n<p><b>History:</b> Point Alpha and its contents are believed to date back to prehistory. Babylonian scripts indicate that SCP-001-1 was routinely patrolled by a group of sages, known as the "Order of White and Black," who refused passage to anyone who could demonstrate the ability to read or write.</p>\n<p>This group was made defunct by the rise of Achaemenid Persia, which contained SCP-001-1 with a group of Zoroastrian monks. Very few accounts of SCP-001-1 are present; recovered accounts assert that all information about SCP-001-1 and its significance was passed orally.</p>\n<p>The most substantial account of SCP-001-1 and SCP-001-2 at this time dates back to the Greek philosopher Xera, who made an expedition into the Achaemenid Empire and found Point Alpha. During Alexander of Macedon's conquest of the Achaemenid Empire, he took interest in Xera's texts and continued the Achaemenid containment of SCP-001-1. However, this new institution, known as the "Cronus Guard", was given Greek epics to read in order to "punish Cronus" for his actions. This institution lasted through the partition of Macedon into the Seleucid Empire, the rise of the Sassanid Empire, and even the formation of the Rashidun Caliphate.</p>\n<p>The Cronus Guard were eventually replaced with an Islamic equivalent: "The Society for the Containment of the Babel Demon." This group was the first to preform substantial research on SCP-001-1, and the first to accurately link SCP-001 to SCP-001-1.</p>\n<div class="socotbd">\n<p style="font-size: 82%"><a onclick="toggle('socotbd');">View Attachment: Excerpts from the notes of the Society for the Containment of the Babel Demon</a></p>\n</div>\n<div class="socotbd collapsed">\n <p style="font-size: 82%"><a onclick="toggle('socotbd');">\nClose Attachment</a></p>\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/book.jpg" style="width:300px;" alt="book.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Records from the Society for the Containment of the Babel Demon, recovered in the early 19<sup>th</sup> century within ORIA archives.</p>\n</div>\n</div>\n<blockquote>\n<p><b>Literature Read:</b> The first quatrain of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON squirms but otherwise does not react. Consistent with reading of Homer's Epics by the Chronus Guard.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> The remaining quatrains of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> No changes from observed behavior.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> A poem written by Guardian Muhammad ibn Buya'aa meant explicitly for DEMON</p>\n<p><b>Result:</b> DEMON thrashes more than usual until the cessation of the reading. ORB begins to expand until the <i>Genealogies of the Nobles</i> is read to DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> None, for 10 days</p>\n<p><b>Result:</b> ORB is observed to expand, and DEMON is observed to thrash. Externally, the sky above the camp is noted to turn a darker red color, until the <i>Rubaiyat of Omar Khayyam</i> is read again.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON thrashes more than during previous readings of the <i>Rubaiyat of Omar Khayyam</i>. It is believed that the Quran may no longer have any effect on DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>One Thousand and One Nights</i></p>\n<p><b>Result:</b> DEMON ceases thrashing. New literature may be necessary to prevent further phenomena from occurring.</p>\n</blockquote>\n</div>\n<p>The Society was eventually absorbed into the Office for the Reclamation of Islamic Artifacts, who assumed containment of SCP-001-1. After Incident 001-EXAL, the Foundation purchased Point Alpha from ORIA for a large sum of currency and several Safe-class anomalies.</p>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Incident 001-EXAL</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<p style="font-size: 82%"><i>Note: Some documentation taken from ORIA's account of Incident 001-EXAL.</i></p>\n<p>On <span id="today"></span>, personnel with ORIA read the book <i>Tafsir al-Ahlam al-kabir</i>, or "Great Book of Interpretation of Dreams" as per normal containment of SCP-001-1. However, SCP-001-2 was observed to rapidly thrash while screaming. SCP-001-1 itself swelled from 50.3 centimeters to 3.2 meters in diameter.</p>\n<p>Shortly following this event, SCP-001 began to worsen. The sky worldwide turned a black/white color similar to the texture of SCP-001-1. In addition, reality bending phenomena began worldwide, causing deformed geography, the manifestation of dangerous anomalous objects and several natural disasters. This event was ended after ORIA personnel read SCP-001-1 an undisclosed number of as-of-yet unread books, which caused SCP-001-2 to stop thrashing and SCP-001 to restore to its pre-incident point. However, the damage caused by this incident was deemed enough to require an activation of <a target="_top" href="http://www.scp-wiki.net/scp-2000">SCP-2000</a>.</p>\n<p>This prompted the Foundation to take control of containment of SCP-001. See <b>History</b> segment above for more information.</p>\n<div class="info">\n<p style="font-size: 82%"><a onclick="toggle('info');">Input Level 5/001 Credentials</a></p>\n</div>\n<div class="info collapsed">\n <p style="font-size: 82%"><a onclick="toggle('info');">\nAccess Granted</a></p>\n<p>During Incident 001-Alpha, vocalizations in <a target="_top" href="http://www.scp-wiki.net/scp-140">Daevish</a> were recorded by ORIA instruments within Point Alpha. The source is presently unknown. The following are approximate translations of these vocalizations.</p>\n<blockquote>\n<p>It has passed some [space/time].</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. It [has had/used to be] fun. It is [time/space] to leave.</p>\n</blockquote>\n<blockquote>\n<p>You cannot [stay/root/hold] yourself forever.</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. You [must/will] awaken.</p>\n</blockquote>\n<blockquote>\n<p>No, you have [entered/reentered] slumber for too long. Wake up.</p>\n</blockquote>\n<blockquote>\n<p>[King/Prince/loved one], it is time for you to wake up.</p>\n</blockquote>\n<blockquote>\n<p>It is fun to be in the [UNKNOWN: gaera] but you cannot be [in/rooted] there [forever/unending]. It is [entertaining/deathlike], but you must wake up.</p>\n</blockquote>\n<blockquote>\n<p>Wake up, [King/Prince/loved one]. We miss you.</p>\n</blockquote>\n</div>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Addenda</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<div class="discus">\n<p style="font-size: 82%"><a onclick="toggle('discus');">Input Level 5/001 Clearance</a></p>\n</div>\n<div class="discus collapsed">\n <p style="font-size: 82%"><a onclick="toggle('discus');">\nAccess Granted</a></p>\n<div div class="content-panel standalone series">\n<center>\n<h2>Discussion Thread 001-398:<br />Continued Containment</h2>\n<p style="font-size: 82%">Started on: <span id="today2"></span></p>\n<p style="font-size: 82%">Started by: <tt>HMCL Robinson</tt></p>\n</center>\n</div>\n<blockquote>\n<p><b><tt>HMCL Robinson:</tt></b> The usage of this thread will be for containment directives for SCP-001. In the past year that we have spent containing it, SCP-001-1's literature needs have become increasingly draconic: we can very rarely reread books without it beginning to thrash, and even then we're running out of high-quality literature to give it. We need a more long-term, reliable solution for containment. SCP-001's file has been temporarily opened up to all personnel with Level 2 clearance. Anybody can submit an idea.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Use Artificial Intelligence Constructs to automatically generate new stories for SCP-001-2.</p>\n<p><b>Status: <tt>[ <span class="green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> TSATPWTCOTTTADC.aic was able to generate 10,000 volumes of stories imitating Greek literature. However, when the first was read to SCP-001-1, it began thrashing beyond acceptable measures until it was read approved reading. No further computer-generated works are to be given to SCP-001-1.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Hire a full team of authors to create literature for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> In the past, literature written specifically for SCP-001-1 have failed to contain it (see: SCtBD Document #249). With SCP-001-2's current state this is not to be attempted again.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> This could represent a possible information leak. Test denied.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Reason:</b> With SCP-001-2's recent containment breach, these measures are approved.</p>\n<p><b>Result:</b> SCP-001-2 observed to stop thrashing and stay completely still, seemingly smiling. In addition, SCP-███ was able to be read for 14 consecutive readings before SCP-001-2 resumed normal activity.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Regularly read mainlist documentation for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Replacement of the SCP-001 file with several "001 Proposals" with much more grand implications than normal SCP files.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> By only occasionally reading 001 files, SCP-001-2 is observed to be more calm on average. The previously observed actions of retaliation in response to specially crafted literature do not apply here.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Given the variation of the current 2957 SCP mainlist files, SCP-001-1 is to be read a random arrangement of these files, in repeat.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> This strategy succeeded for approximately 3 years, allowing for several repeats. However, on <span id="yearplus3"></span>-9-30, SCP-001-2 began thrashing rapidly, requiring newer documentation to sedate.</p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-4999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> SCP-001-1 has required progressively more SCP mainlist articles for containment, even to the point of the fabrication of some anomalous entities.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Rewriting of several older SCP mainlist files to be more exaggerated and narrative-like.</p>\n<p><b>Status: <tt>[ <span style="color: orange">TENTATIVELY APPROVED</span> ]</tt></b></p>\n<p><b>Note:</b> <i>Deliberately exaggerating our files for the purpose of this entity is crossing some lines, but for now, it will have to do. However, we need a better way. I'm calling a committee. We need to put a stop to this.</i></p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Creation of a file describing SCP-001 that uses recursion to effectively be "never-ending." Psychoanalysis related to SCP-001-2's apparent "enjoyment" of works implies a slight ego and bias towards works about itself, and past strategies with this idea have worked for long periods of time. File is to be archived as a "technical" file.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> Procedure 001-ENTRY implemented. [DATA EXPUNGED]</p>\n</blockquote>\n</div>\n\n<div class="newiter">\n<p style="font-size: 82%"><a onclick="new_iteration();">Input Level 5/TECHNICAL Clearance</a></p>\n</div>\n<div class="inner">\n<!-- :-) -->\n</div>\n</div>\n</body>\n[[/code]]
259	[[>]]\n[[module Rate]]\n[[/>]]\n[[code type="html"]]\n<body>\n<style type="text/css">\nbody\n{\n   font-family:verdana,arial,helvetica,sans-serif;\n   font-size:12.8px;\n   color:#333;\n   line-height:141%\n}\nblockquote\n{\n  border:1px dashed #999;\n  padding: 0 12.8px;\n  background-color:#f4f4f4\n}\na {\n    color: #b01;\n    text-decoration: none;\n    background: transparent;\n}\n.content-panel {\n    border: solid 1px #888880;\n    border-radius: 10px;\n    background-color: #999990;\n    margin: 10px 0 15px;\n    box-shadow: 3px 3px 6px #bbb;\n    box-shadow: 0 2px 6px rgba(0,0,0,0.5), inset 0 1px rgba(255,255,255,0.3), inset 0 10px rgba(255,255,255,0.2), inset 0 10px 20px rgba(255,255,255,0.25), inset 0 -15px 30px rgba(0,0,0,0.1);\n}\n.content-panel.standalone {\n    background: #fcfdfb;\n}\n.content-panel.series {\n    padding: 0 20px;\n    margin-bottom: 20px;\n}\na.newpage {\n    color: #d61;\n    text-decoration: none;\n    background: transparent;\n}\na:hover {\n    text-decoration: underline;\n    background-color: transparent;\n}\n.collapsed {\n  display: none;\n}\n.vanished {\n  display: none;\n}\n.warning-cont {\n  color: red;\n  text-align: center;\n}\n#warning-header {\n  font-size: 25.6px;\n}\n#warning {\n  font-size: 20px;\n}\n.retrofont {\nfont-family: 'Geo', cursive;\nfont-weight: bold;\n}\n.cblwarning {\n  color: red;\n}\n.green {\n  color: green;\n}\n.smaller-g {\nwidth: 45%;\nmargin: auto;\nbackground-color: #e6e6e6;\nborder: none;\nheight: 2px;\n}\n.adden{\nfont-size: 1.3em;\n}\n/* Standard Image Block */\n.scp-image-block {\n    border: solid 1px #666;\n    box-shadow: 0 1px 6px rgba(0,0,0,.25);\n    width: 300px;\n}\n \n.scp-image-block.block-right {\n        float: right;\n    clear: right;\n    margin: 0 2em 1em 2em;\n}\n \n.scp-image-block.block-left {\n    float: left;\n    clear: left;\n    margin: 0 2em 1em 0;\n}\n \n.scp-image-block.block-center {\n    margin-right: auto;\n    margin-left: auto;\n}\n.scp-image-block img {\n    border: 0;\n    width: 300px;\n}\n.scp-image-block .scp-image-caption {\n    background-color: #eee;\n    border-top: solid 1px #666;\n    padding: 2px 0;\n    font-size: 80%;\n    font-weight: bold;\n    text-align: center;\n    width: 300px;\n}\n.scp-image-block > p {\n    margin: 0;\n}\n.scp-image-block .scp-image-caption > p {\n    margin: 0;\n    padding: 0 10px;\n    line-height: 125%\n}\n</style>\n<script type="text/javascript">\nfunction addEvent(element, eventName, callback) {\n    if (element.addEventListener) {\n        element.addEventListener(eventName, callback, false);\n    } else if (element.attachEvent) {\n        element.attachEvent("on" + eventName, callback);\n    } else {\n        element["on" + eventName] = callback;\n    }\n}\n\nwindow.toggle = function(cls) {\n  var collapsed = document.querySelectorAll("." + cls + ".collapsed");\n  var expanded = document.querySelectorAll("." + cls + ":not(.collapsed)");\n  var i;\n  for(i = 0; i < collapsed.length; i++) {\n    collapsed[i].classList.remove("collapsed");\n  }\n  for(i = 0; i < expanded.length; i++) {\n    expanded[i].classList.add("collapsed");\n  }\n};\n\n// typewriter code\nfunction Typewriter(el, str, delay) {\n  if (!(this instanceof Typewriter)) return new Typewriter(el, str, delay);\n\n  this.el = el;\n  this.str = str || el.innerHTML || el.value;\n  this.delay = delay || 100;\n  this.i = 0;\n}\n\nTypewriter.prototype.type = function() {\n  var i = this.i,\n    char = this.str.charAt(i);\n\n  if (!char) {\n    clearInterval(this.intervalID);\n    return this;\n  }\n\n  if (char === '<') this.isTag = true;\n  if (char === '>') this.isTag = false;\n\n  this.el.innerHTML += char;\n  return this.i++;\n};\n\nTypewriter.prototype.start = function() {\n  var self = this;\n  if (this.i < 0) this.i = 0;\n  if (this.el.innerHTML === this.str) this.clear();\n\n  // this.emit('start');\n  (function loop() {\n    self.type();\n    if (self.isTag) return loop();\n    self.intervalID = setTimeout(loop, self.delay);\n  }());\n\n  return this;\n};\n\nTypewriter.prototype.stop = function() {\n  this.i = -1;\n  return this;\n};\n\nTypewriter.prototype.restart = function() {\n  this.clear();\n  this.i = 0;\n  return this.start();\n};\n\nTypewriter.prototype.clear = function() {\n  this.el.innerHTML = '';\n  return this;\n};\n\nvar junkChars = ['ó','ò','ñ','ç','¿','ß','à','è','ì','ù','Œ','þ','█','%','+','=','.','«','»','£','$','€','½','¾','À','È','Ì','Ò','Ù','Â','Ê','â','ê','Ä','Ë','ä','ë','ÿ','ü','•'];\n\nvar an_iteration = "";\n\n\nvar randomIntFromInterval = function(min,max)\n{\n  return Math.floor(Math.random()*(max-min+1)+min);\n};\n\nvar getJunkChars = function(length) {\n  var res = '';\n  var i;\n  for (i = 0; i < length; i++) {\n    res += junkChars[randomIntFromInterval(0,junkChars.length - 1)];\n  }\n  return res;\n};\n\nvar selTabs = [];\n\nfunction rndiam() {\n  var rndiams = document.getElementsByClassName('rndiam');\n  var i;\n  for (i = 0; i < rndiams.length; i++) {\n    rndiams[i].innerHTML = (Math.random() * (1.56 - 0.51) + 0.51).toFixed(2);\n  }\n  setTimeout(rndiam, 2000);\n}\n\nwindow.onload = function() {\n\nselTabs = [\n  document.getElementsByClassName('selTab1')[0],\n  document.getElementsByClassName('selTab2')[0],\n  document.getElementsByClassName('selTab3')[0],\n  document.getElementsByClassName('selTab4')[0],\n  document.getElementsByClassName('selTab5')[0]\n];\n\nvar msg1 = '= SCP-001 | Technical Clearance Required =';\n\nvar delay1 = 50;\nvar delay2 = 25;\n\nvar toD1 = delay1 * (msg1.length);\n\nvar stopEarly = function(){\n  document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished");\n};\n\nvar typ1 =Typewriter(document.querySelectorAll('#warning-header')[0],msg1,delay1)\n\ntyp1.start();\nsetTimeout(function() { document.getElementById("warning").classList.remove("vanished"); }, toD1 + 500);\nsetTimeout(function () { document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished"); }, toD1 + 1500);\n\n  rndiam();\n\n  var today = new Date();\n  var todayformat = "" + (today.getFullYear() - 1) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today").innerHTML = todayformat;\n  var todayformat2 = "" + (today.getFullYear() - 0) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today2").innerHTML = todayformat2;\n  document.getElementById("yearplus3").innerHTML = "" + (today.getFullYear() + 3);\n\n  an_iteration = document.getElementById("part1").innerHTML;\n};\n\nwindow.new_iteration = function() {\n  var new_iters = document.getElementsByClassName("newiter");\n  console.log(new_iters);\n  new_iters[new_iters.length - 1].classList.add("vanished");\n  var inner = document.getElementsByClassName("inner");\n  inner[inner.length - 1].innerHTML = "<hr /><hr />" + an_iteration;\n};\n</script>\n</script>\n<div class="warning-cont">\n<h2 id="warning-header">\n<p>\n</p>\n<h3 id="warning" class="vanished">  \n<p>This document exists as technical containment for an anomaly and thus does not adhere to standard SCiPNET formatting. Input 5/TECHNICAL clearance to proceed.\n</p>\n</div>\n\n<div id="collapsible_to_open_skip" class="article vanished">\n<p><center><a onclick="toggle('article');">[ INPUT PERSONAL IDENTIFICATION NUMBER ]</a></center></p>\n</div>\n<div class="article collapsed">\n <p><center><a onclick="toggle('article');">[ PERSONAL IDENTIFICATION NUMBER VERIFIED ]</a></center></p>\n<div id="part1">\n<hr />\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://topia.wikidot.com/local--files/jamcon-001/sky.jpg" style="width:300px;" alt="destruction.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>The sky above Point Alpha during a breach of SCP-001.</p>\n</div>\n</div>\n<p><b>Item #:</b> SCP-001</p>\n<p><b>Object Class:</b> Ontokinetic</p>\n<p><b>Containment Class:</b> Keter</p>\n<p><b>Special Containment Procedures:</b> If the sky begins changing color with no prior stimuli or indication, all Foundation sites are to go onto ALERT LEVEL 7 immediately. Personnel at Site-05 and -06 are to enter ALERT LEVEL 8 and begin evaluating new methods of containing SCP-001.</p>\n<p>Site-05 has been constructed around Point Alpha. Procedure 001-ENTRY is in effect to contain SCP-001-1. A monitor containing Document 032, as well as a speaker reading the document, are to be placed at least five meters away from SCP-001-1. If SCP-001-1 swells up to 2 meters or more in diameter, the speed of the narration is to increase. This system is to have a direct cable link to the Foundation SCiPNET database at Site-06, and several redundant power generators are to ensure this system stays operational at all times.</p>\n<p><b>Description:</b> SCP-001 is an ongoing ZK-Class "Reality Failure" Scenario. SCP-001 is prevented from happening through several procedures manipulating its sub-anomalies. The only known observable symptom of SCP-001 is the sky becoming a different color; it is believed that, if SCP-001 were to progress to the point where other symptoms began to occur, it would become irreversible.</p>\n<p>SCP-001-1 is a levitating object contained within Point Alpha, a cave chamber five kilometers south of the ruins of Babylon. SCP-001-1 visually resembles a perfectly smooth sphere with a texture similar to blurred television static. Measurement instruments indicate SCP-001-1 is currently <span class="rndiam"></span> meters in diameter. Solids and liquids that pass within the bounds of SCP-001-1 are replaced entirely with argon gas, giving the illusion of disappearing.</p>\n<p>SCP-001-2 is an entity that is capable of being seen within Point Alpha. SCP-001-2's manifestations are random; witnesses report seeing SCP-001-2 "past the walls of the cave." From descriptions of eyewitnesses, SCP-001-2 is humanoid in shape and large in stature, and is only visible when looking through SCP-001-1. Descriptions of SCP-001-2 vary from person to person; common features include an emaciated figure, long limbs, and prominent facial features. SCP-001-2 takes the position of lying against the wall. It is most often seen with a neutral expression; however, the expression occasionally changes to discomfort or bliss.</p>\n<p>It has been determined from historical precedent that "entertaining" SCP-001-2 through SCP-001-1 will prevent SCP-001.</p>\n<div class="scp-image-block block-left" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/cave.jpg" style="width:300px;" alt="cave.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Point Alpha shortly after it was acquired by the Foundation, with Agents Samuel and Boise preforming preliminary evaluation. SCP-001-1 not visible.</p>\n</div>\n</div>\n<p><b>History:</b> Point Alpha and its contents are believed to date back to prehistory. Babylonian scripts indicate that SCP-001-1 was routinely patrolled by a group of sages, known as the "Order of White and Black," who refused passage to anyone who could demonstrate the ability to read or write.</p>\n<p>This group was made defunct by the rise of Achaemenid Persia, which contained SCP-001-1 with a group of Zoroastrian monks. Very few accounts of SCP-001-1 are present; recovered accounts assert that all information about SCP-001-1 and its significance was passed orally.</p>\n<p>The most substantial account of SCP-001-1 and SCP-001-2 at this time dates back to the Greek philosopher Xera, who made an expedition into the Achaemenid Empire and found Point Alpha. During Alexander of Macedon's conquest of the Achaemenid Empire, he took interest in Xera's texts and continued the Achaemenid containment of SCP-001-1. However, this new institution, known as the "Cronus Guard", was given Greek epics to read in order to "punish Cronus" for his actions. This institution lasted through the partition of Macedon into the Seleucid Empire, the rise of the Sassanid Empire, and even the formation of the Rashidun Caliphate.</p>\n<p>The Cronus Guard were eventually replaced with an Islamic equivalent: "The Society for the Containment of the Babel Demon." This group was the first to preform substantial research on SCP-001-1, and the first to accurately link SCP-001 to SCP-001-1.</p>\n<div class="socotbd">\n<p style="font-size: 82%"><a onclick="toggle('socotbd');">View Attachment: Excerpts from the notes of the Society for the Containment of the Babel Demon</a></p>\n</div>\n<div class="socotbd collapsed">\n <p style="font-size: 82%"><a onclick="toggle('socotbd');">\nClose Attachment</a></p>\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/book.jpg" style="width:300px;" alt="book.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Records from the Society for the Containment of the Babel Demon, recovered in the early 19<sup>th</sup> century within ORIA archives.</p>\n</div>\n</div>\n<blockquote>\n<p><b>Literature Read:</b> The first quatrain of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON squirms but otherwise does not react. Consistent with reading of Homer's Epics by the Chronus Guard.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> The remaining quatrains of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> No changes from observed behavior.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> A poem written by Guardian Muhammad ibn Buya'aa meant explicitly for DEMON</p>\n<p><b>Result:</b> DEMON thrashes more than usual until the cessation of the reading. ORB begins to expand until the <i>Genealogies of the Nobles</i> is read to DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> None, for 10 days</p>\n<p><b>Result:</b> ORB is observed to expand, and DEMON is observed to thrash. Externally, the sky above the camp is noted to turn a darker red color, until the <i>Rubaiyat of Omar Khayyam</i> is read again.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON thrashes more than during previous readings of the <i>Rubaiyat of Omar Khayyam</i>. It is believed that the Quran may no longer have any effect on DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>One Thousand and One Nights</i></p>\n<p><b>Result:</b> DEMON ceases thrashing. New literature may be necessary to prevent further phenomena from occurring.</p>\n</blockquote>\n</div>\n<p>The Society was eventually absorbed into the Office for the Reclamation of Islamic Artifacts, who assumed containment of SCP-001-1. After Incident 001-EXAL, the Foundation purchased Point Alpha from ORIA for a large sum of currency and several Safe-class anomalies.</p>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Incident 001-EXAL</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<p style="font-size: 82%"><i>Note: Some documentation taken from ORIA's account of Incident 001-EXAL.</i></p>\n<p>On <span id="today"></span>, personnel with ORIA read the book <i>Tafsir al-Ahlam al-kabir</i>, or "Great Book of Interpretation of Dreams" as per normal containment of SCP-001-1. However, SCP-001-2 was observed to rapidly thrash while screaming. SCP-001-1 itself swelled from 50.3 centimeters to 3.2 meters in diameter.</p>\n<p>Shortly following this event, SCP-001 began to worsen. The sky worldwide turned a black/white color similar to the texture of SCP-001-1. In addition, reality bending phenomena began worldwide, causing deformed geography, the manifestation of dangerous anomalous objects and several natural disasters. This event was ended after ORIA personnel read SCP-001-1 an undisclosed number of as-of-yet unread books, which caused SCP-001-2 to stop thrashing and SCP-001 to restore to its pre-incident point. However, the damage caused by this incident was deemed enough to require an activation of <a target="_top" href="http://www.scp-wiki.net/scp-2000">SCP-2000</a>.</p>\n<p>This prompted the Foundation to take control of containment of SCP-001. See <b>History</b> segment above for more information.</p>\n<div class="info">\n<p style="font-size: 82%"><a onclick="toggle('info');">Input Level 5/001 Credentials</a></p>\n</div>\n<div class="info collapsed">\n <p style="font-size: 82%"><a onclick="toggle('info');">\nAccess Granted</a></p>\n<p>During Incident 001-Alpha, vocalizations in <a target="_top" href="http://www.scp-wiki.net/scp-140">Daevish</a> were recorded by ORIA instruments within Point Alpha. The source is presently unknown. The following are approximate translations of these vocalizations.</p>\n<blockquote>\n<p>It has passed some [space/time].</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. It [has had/used to be] fun. It is [time/space] to leave.</p>\n</blockquote>\n<blockquote>\n<p>You cannot [stay/root/hold] yourself forever.</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. You [must/will] awaken.</p>\n</blockquote>\n<blockquote>\n<p>No, you have [entered/reentered] slumber for too long. Wake up.</p>\n</blockquote>\n<blockquote>\n<p>[King/Prince/loved one], it is time for you to wake up.</p>\n</blockquote>\n<blockquote>\n<p>It is fun to be in the [UNKNOWN: gaera] but you cannot be [in/rooted] there [forever/unending]. It is [entertaining/deathlike], but you must wake up.</p>\n</blockquote>\n<blockquote>\n<p>Wake up, [King/Prince/loved one]. We miss you.</p>\n</blockquote>\n</div>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Addenda</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<div class="discus">\n<p style="font-size: 82%"><a onclick="toggle('discus');">Input Level 5/001 Clearance</a></p>\n</div>\n<div class="discus collapsed">\n <p style="font-size: 82%"><a onclick="toggle('discus');">\nAccess Granted</a></p>\n<div div class="content-panel standalone series">\n<center>\n<h2>Discussion Thread 001-398:<br />Continued Containment</h2>\n<p style="font-size: 82%">Started on: <span id="today2"></span></p>\n<p style="font-size: 82%">Started by: <tt>HMCL Robinson</tt></p>\n</center>\n</div>\n<blockquote>\n<p><b><tt>HMCL Robinson:</tt></b> The usage of this thread will be for containment directives for SCP-001. In the past year that we have spent containing it, SCP-001-1's literature needs have become increasingly draconic: we can very rarely reread books without it beginning to thrash, and even then we're running out of high-quality literature to give it. We need a more long-term, reliable solution for containment. SCP-001's file has been temporarily opened up to all personnel with Level 2 clearance. Anybody can submit an idea.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Use Artificial Intelligence Constructs to automatically generate new stories for SCP-001-2.</p>\n<p><b>Status: <tt>[ <span class="green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> TSATPWTCOTTTADC.aic was able to generate 10,000 volumes of stories imitating Greek literature. However, when the first was read to SCP-001-1, it began thrashing beyond acceptable measures until it was read approved reading. No further computer-generated works are to be given to SCP-001-1.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Hire a full team of authors to create literature for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> In the past, literature written specifically for SCP-001-1 have failed to contain it (see: SCtBD Document #249). With SCP-001-2's current state this is not to be attempted again.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> This could represent a possible information leak. Test denied.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Reason:</b> With SCP-001-2's recent containment breach, these measures are approved.</p>\n<p><b>Result:</b> SCP-001-2 observed to stop thrashing and stay completely still, seemingly smiling. In addition, SCP-███ was able to be read for 14 consecutive readings before SCP-001-2 resumed normal activity.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Regularly read mainlist documentation for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Replacement of the SCP-001 file with several "001 Proposals" with much more grand implications than normal SCP files.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> By only occasionally reading 001 files, SCP-001-2 is observed to be more calm on average. The previously observed actions of retaliation in response to specially crafted literature do not apply here.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Given the variation of the current 2957 SCP mainlist files, SCP-001-1 is to be read a random arrangement of these files, in repeat.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> This strategy succeeded for approximately 3 years, allowing for several repeats. However, on <span id="yearplus3"></span>-9-30, SCP-001-2 began thrashing rapidly, requiring newer documentation to sedate.</p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-4999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> SCP-001-1 has required progressively more SCP mainlist articles for containment, even to the point of the fabrication of some anomalous entities.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Rewriting of several older SCP mainlist files to be more exaggerated and narrative-like.</p>\n<p><b>Status: <tt>[ <span style="color: orange">TENTATIVELY APPROVED</span> ]</tt></b></p>\n<p><b>Note:</b> <i>Deliberately exaggerating our files for the purpose of this entity is crossing some lines, but for now, it will have to do. However, we need a better way. I'm calling a committee. We need to put a stop to this.</i></p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Creation of a file describing SCP-001 that uses recursion to effectively be "never-ending." Psychoanalysis related to SCP-001-2's apparent "enjoyment" of works implies a slight ego and bias towards works about itself, and past strategies with this idea have worked for long periods of time. File is to be archived as a "technical" file.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> Procedure 001-ENTRY implemented. [DATA EXPUNGED]</p>\n</blockquote>\n</div>\n\n<div class="newiter">\n<p style="font-size: 82%"><a onclick="new_iteration();">Input Level 5/TECHNICAL Clearance</a></p>\n</div>\n<div class="inner">\n<!-- :-) -->\n</div>\n</div>\n</body>\n[[/code]]
260	[[>]]\n[[module Rate]]\n[[/>]]\n[[code type="html"]]\n<html>\n<body>\n<style type="text/css">\nbody\n{\n   font-family:verdana,arial,helvetica,sans-serif;\n   font-size:12.8px;\n   color:#333;\n   line-height:141%\n}\nblockquote\n{\n  border:1px dashed #999;\n  padding: 0 12.8px;\n  background-color:#f4f4f4\n}\na {\n    color: #b01;\n    text-decoration: none;\n    background: transparent;\n}\n.content-panel {\n    border: solid 1px #888880;\n    border-radius: 10px;\n    background-color: #999990;\n    margin: 10px 0 15px;\n    box-shadow: 3px 3px 6px #bbb;\n    box-shadow: 0 2px 6px rgba(0,0,0,0.5), inset 0 1px rgba(255,255,255,0.3), inset 0 10px rgba(255,255,255,0.2), inset 0 10px 20px rgba(255,255,255,0.25), inset 0 -15px 30px rgba(0,0,0,0.1);\n}\n.content-panel.standalone {\n    background: #fcfdfb;\n}\n.content-panel.series {\n    padding: 0 20px;\n    margin-bottom: 20px;\n}\na.newpage {\n    color: #d61;\n    text-decoration: none;\n    background: transparent;\n}\na:hover {\n    text-decoration: underline;\n    background-color: transparent;\n}\n.collapsed {\n  display: none;\n}\n.vanished {\n  display: none;\n}\n.warning-cont {\n  color: red;\n  text-align: center;\n}\n#warning-header {\n  font-size: 25.6px;\n}\n#warning {\n  font-size: 20px;\n}\n.retrofont {\nfont-family: 'Geo', cursive;\nfont-weight: bold;\n}\n.cblwarning {\n  color: red;\n}\n.green {\n  color: green;\n}\n.smaller-g {\nwidth: 45%;\nmargin: auto;\nbackground-color: #e6e6e6;\nborder: none;\nheight: 2px;\n}\n.adden{\nfont-size: 1.3em;\n}\n/* Standard Image Block */\n.scp-image-block {\n    border: solid 1px #666;\n    box-shadow: 0 1px 6px rgba(0,0,0,.25);\n    width: 300px;\n}\n \n.scp-image-block.block-right {\n        float: right;\n    clear: right;\n    margin: 0 2em 1em 2em;\n}\n \n.scp-image-block.block-left {\n    float: left;\n    clear: left;\n    margin: 0 2em 1em 0;\n}\n \n.scp-image-block.block-center {\n    margin-right: auto;\n    margin-left: auto;\n}\n.scp-image-block img {\n    border: 0;\n    width: 300px;\n}\n.scp-image-block .scp-image-caption {\n    background-color: #eee;\n    border-top: solid 1px #666;\n    padding: 2px 0;\n    font-size: 80%;\n    font-weight: bold;\n    text-align: center;\n    width: 300px;\n}\n.scp-image-block > p {\n    margin: 0;\n}\n.scp-image-block .scp-image-caption > p {\n    margin: 0;\n    padding: 0 10px;\n    line-height: 125%\n}\n</style>\n<script type="text/javascript">\nfunction addEvent(element, eventName, callback) {\n    if (element.addEventListener) {\n        element.addEventListener(eventName, callback, false);\n    } else if (element.attachEvent) {\n        element.attachEvent("on" + eventName, callback);\n    } else {\n        element["on" + eventName] = callback;\n    }\n}\n\nwindow.toggle = function(cls) {\n  var collapsed = document.querySelectorAll("." + cls + ".collapsed");\n  var expanded = document.querySelectorAll("." + cls + ":not(.collapsed)");\n  var i;\n  for(i = 0; i < collapsed.length; i++) {\n    collapsed[i].classList.remove("collapsed");\n  }\n  for(i = 0; i < expanded.length; i++) {\n    expanded[i].classList.add("collapsed");\n  }\n};\n\n// typewriter code\nfunction Typewriter(el, str, delay) {\n  if (!(this instanceof Typewriter)) return new Typewriter(el, str, delay);\n\n  this.el = el;\n  this.str = str || el.innerHTML || el.value;\n  this.delay = delay || 100;\n  this.i = 0;\n}\n\nTypewriter.prototype.type = function() {\n  var i = this.i,\n    char = this.str.charAt(i);\n\n  if (!char) {\n    clearInterval(this.intervalID);\n    return this;\n  }\n\n  if (char === '<') this.isTag = true;\n  if (char === '>') this.isTag = false;\n\n  this.el.innerHTML += char;\n  return this.i++;\n};\n\nTypewriter.prototype.start = function() {\n  var self = this;\n  if (this.i < 0) this.i = 0;\n  if (this.el.innerHTML === this.str) this.clear();\n\n  // this.emit('start');\n  (function loop() {\n    self.type();\n    if (self.isTag) return loop();\n    self.intervalID = setTimeout(loop, self.delay);\n  }());\n\n  return this;\n};\n\nTypewriter.prototype.stop = function() {\n  this.i = -1;\n  return this;\n};\n\nTypewriter.prototype.restart = function() {\n  this.clear();\n  this.i = 0;\n  return this.start();\n};\n\nTypewriter.prototype.clear = function() {\n  this.el.innerHTML = '';\n  return this;\n};\n\nvar junkChars = ['ó','ò','ñ','ç','¿','ß','à','è','ì','ù','Œ','þ','█','%','+','=','.','«','»','£','$','€','½','¾','À','È','Ì','Ò','Ù','Â','Ê','â','ê','Ä','Ë','ä','ë','ÿ','ü','•'];\n\nvar an_iteration = "";\n\n\nvar randomIntFromInterval = function(min,max)\n{\n  return Math.floor(Math.random()*(max-min+1)+min);\n};\n\nvar getJunkChars = function(length) {\n  var res = '';\n  var i;\n  for (i = 0; i < length; i++) {\n    res += junkChars[randomIntFromInterval(0,junkChars.length - 1)];\n  }\n  return res;\n};\n\nvar selTabs = [];\n\nfunction rndiam() {\n  var rndiams = document.getElementsByClassName('rndiam');\n  var i;\n  for (i = 0; i < rndiams.length; i++) {\n    rndiams[i].innerHTML = (Math.random() * (1.56 - 0.51) + 0.51).toFixed(2);\n  }\n  setTimeout(rndiam, 2000);\n}\n\nwindow.onload = function() {\n\nselTabs = [\n  document.getElementsByClassName('selTab1')[0],\n  document.getElementsByClassName('selTab2')[0],\n  document.getElementsByClassName('selTab3')[0],\n  document.getElementsByClassName('selTab4')[0],\n  document.getElementsByClassName('selTab5')[0]\n];\n\nvar msg1 = '= SCP-001 | Technical Clearance Required =';\n\nvar delay1 = 50;\nvar delay2 = 25;\n\nvar toD1 = delay1 * (msg1.length);\n\nvar stopEarly = function(){\n  document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished");\n};\n\nvar typ1 =Typewriter(document.querySelectorAll('#warning-header')[0],msg1,delay1)\n\ntyp1.start();\nsetTimeout(function() { document.getElementById("warning").classList.remove("vanished"); }, toD1 + 500);\nsetTimeout(function () { document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished"); }, toD1 + 1500);\n\n  rndiam();\n\n  var today = new Date();\n  var todayformat = "" + (today.getFullYear() - 1) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today").innerHTML = todayformat;\n  var todayformat2 = "" + (today.getFullYear() - 0) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today2").innerHTML = todayformat2;\n  document.getElementById("yearplus3").innerHTML = "" + (today.getFullYear() + 3);\n\n  an_iteration = document.getElementById("part1").innerHTML;\n};\n\nwindow.new_iteration = function() {\n  var new_iters = document.getElementsByClassName("newiter");\n  console.log(new_iters);\n  new_iters[new_iters.length - 1].classList.add("vanished");\n  var inner = document.getElementsByClassName("inner");\n  inner[inner.length - 1].innerHTML = "<hr /><hr />" + an_iteration;\n};\n</script>\n</script>\n<div class="warning-cont">\n<h2 id="warning-header">\n<p>\n</p>\n<h3 id="warning" class="vanished">  \n<p>This document exists as technical containment for an anomaly and thus does not adhere to standard SCiPNET formatting. Input 5/TECHNICAL clearance to proceed.\n</p>\n</div>\n\n<div id="collapsible_to_open_skip" class="article vanished">\n<p><center><a onclick="toggle('article');">[ INPUT PERSONAL IDENTIFICATION NUMBER ]</a></center></p>\n</div>\n<div class="article collapsed">\n <p><center><a onclick="toggle('article');">[ PERSONAL IDENTIFICATION NUMBER VERIFIED ]</a></center></p>\n<div id="part1">\n<hr />\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://topia.wikidot.com/local--files/jamcon-001/sky.jpg" style="width:300px;" alt="destruction.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>The sky above Point Alpha during a breach of SCP-001.</p>\n</div>\n</div>\n<p><b>Item #:</b> SCP-001</p>\n<p><b>Object Class:</b> Ontokinetic</p>\n<p><b>Containment Class:</b> Keter</p>\n<p><b>Special Containment Procedures:</b> If the sky begins changing color with no prior stimuli or indication, all Foundation sites are to go onto ALERT LEVEL 7 immediately. Personnel at Site-05 and -06 are to enter ALERT LEVEL 8 and begin evaluating new methods of containing SCP-001.</p>\n<p>Site-05 has been constructed around Point Alpha. Procedure 001-ENTRY is in effect to contain SCP-001-1. A monitor containing Document 032, as well as a speaker reading the document, are to be placed at least five meters away from SCP-001-1. If SCP-001-1 swells up to 2 meters or more in diameter, the speed of the narration is to increase. This system is to have a direct cable link to the Foundation SCiPNET database at Site-06, and several redundant power generators are to ensure this system stays operational at all times.</p>\n<p><b>Description:</b> SCP-001 is an ongoing ZK-Class "Reality Failure" Scenario. SCP-001 is prevented from happening through several procedures manipulating its sub-anomalies. The only known observable symptom of SCP-001 is the sky becoming a different color; it is believed that, if SCP-001 were to progress to the point where other symptoms began to occur, it would become irreversible.</p>\n<p>SCP-001-1 is a levitating object contained within Point Alpha, a cave chamber five kilometers south of the ruins of Babylon. SCP-001-1 visually resembles a perfectly smooth sphere with a texture similar to blurred television static. Measurement instruments indicate SCP-001-1 is currently <span class="rndiam"></span> meters in diameter. Solids and liquids that pass within the bounds of SCP-001-1 are replaced entirely with argon gas, giving the illusion of disappearing.</p>\n<p>SCP-001-2 is an entity that is capable of being seen within Point Alpha. SCP-001-2's manifestations are random; witnesses report seeing SCP-001-2 "past the walls of the cave." From descriptions of eyewitnesses, SCP-001-2 is humanoid in shape and large in stature, and is only visible when looking through SCP-001-1. Descriptions of SCP-001-2 vary from person to person; common features include an emaciated figure, long limbs, and prominent facial features. SCP-001-2 takes the position of lying against the wall. It is most often seen with a neutral expression; however, the expression occasionally changes to discomfort or bliss.</p>\n<p>It has been determined from historical precedent that "entertaining" SCP-001-2 through SCP-001-1 will prevent SCP-001.</p>\n<div class="scp-image-block block-left" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/cave.jpg" style="width:300px;" alt="cave.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Point Alpha shortly after it was acquired by the Foundation, with Agents Samuel and Boise preforming preliminary evaluation. SCP-001-1 not visible.</p>\n</div>\n</div>\n<p><b>History:</b> Point Alpha and its contents are believed to date back to prehistory. Babylonian scripts indicate that SCP-001-1 was routinely patrolled by a group of sages, known as the "Order of White and Black," who refused passage to anyone who could demonstrate the ability to read or write.</p>\n<p>This group was made defunct by the rise of Achaemenid Persia, which contained SCP-001-1 with a group of Zoroastrian monks. Very few accounts of SCP-001-1 are present; recovered accounts assert that all information about SCP-001-1 and its significance was passed orally.</p>\n<p>The most substantial account of SCP-001-1 and SCP-001-2 at this time dates back to the Greek philosopher Xera, who made an expedition into the Achaemenid Empire and found Point Alpha. During Alexander of Macedon's conquest of the Achaemenid Empire, he took interest in Xera's texts and continued the Achaemenid containment of SCP-001-1. However, this new institution, known as the "Cronus Guard", was given Greek epics to read in order to "punish Cronus" for his actions. This institution lasted through the partition of Macedon into the Seleucid Empire, the rise of the Sassanid Empire, and even the formation of the Rashidun Caliphate.</p>\n<p>The Cronus Guard were eventually replaced with an Islamic equivalent: "The Society for the Containment of the Babel Demon." This group was the first to preform substantial research on SCP-001-1, and the first to accurately link SCP-001 to SCP-001-1.</p>\n<div class="socotbd">\n<p style="font-size: 82%"><a onclick="toggle('socotbd');">View Attachment: Excerpts from the notes of the Society for the Containment of the Babel Demon</a></p>\n</div>\n<div class="socotbd collapsed">\n <p style="font-size: 82%"><a onclick="toggle('socotbd');">\nClose Attachment</a></p>\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/book.jpg" style="width:300px;" alt="book.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Records from the Society for the Containment of the Babel Demon, recovered in the early 19<sup>th</sup> century within ORIA archives.</p>\n</div>\n</div>\n<blockquote>\n<p><b>Literature Read:</b> The first quatrain of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON squirms but otherwise does not react. Consistent with reading of Homer's Epics by the Chronus Guard.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> The remaining quatrains of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> No changes from observed behavior.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> A poem written by Guardian Muhammad ibn Buya'aa meant explicitly for DEMON</p>\n<p><b>Result:</b> DEMON thrashes more than usual until the cessation of the reading. ORB begins to expand until the <i>Genealogies of the Nobles</i> is read to DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> None, for 10 days</p>\n<p><b>Result:</b> ORB is observed to expand, and DEMON is observed to thrash. Externally, the sky above the camp is noted to turn a darker red color, until the <i>Rubaiyat of Omar Khayyam</i> is read again.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON thrashes more than during previous readings of the <i>Rubaiyat of Omar Khayyam</i>. It is believed that the Quran may no longer have any effect on DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>One Thousand and One Nights</i></p>\n<p><b>Result:</b> DEMON ceases thrashing. New literature may be necessary to prevent further phenomena from occurring.</p>\n</blockquote>\n</div>\n<p>The Society was eventually absorbed into the Office for the Reclamation of Islamic Artifacts, who assumed containment of SCP-001-1. After Incident 001-EXAL, the Foundation purchased Point Alpha from ORIA for a large sum of currency and several Safe-class anomalies.</p>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Incident 001-EXAL</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<p style="font-size: 82%"><i>Note: Some documentation taken from ORIA's account of Incident 001-EXAL.</i></p>\n<p>On <span id="today"></span>, personnel with ORIA read the book <i>Tafsir al-Ahlam al-kabir</i>, or "Great Book of Interpretation of Dreams" as per normal containment of SCP-001-1. However, SCP-001-2 was observed to rapidly thrash while screaming. SCP-001-1 itself swelled from 50.3 centimeters to 3.2 meters in diameter.</p>\n<p>Shortly following this event, SCP-001 began to worsen. The sky worldwide turned a black/white color similar to the texture of SCP-001-1. In addition, reality bending phenomena began worldwide, causing deformed geography, the manifestation of dangerous anomalous objects and several natural disasters. This event was ended after ORIA personnel read SCP-001-1 an undisclosed number of as-of-yet unread books, which caused SCP-001-2 to stop thrashing and SCP-001 to restore to its pre-incident point. However, the damage caused by this incident was deemed enough to require an activation of <a target="_top" href="http://www.scp-wiki.net/scp-2000">SCP-2000</a>.</p>\n<p>This prompted the Foundation to take control of containment of SCP-001. See <b>History</b> segment above for more information.</p>\n<div class="info">\n<p style="font-size: 82%"><a onclick="toggle('info');">Input Level 5/001 Credentials</a></p>\n</div>\n<div class="info collapsed">\n <p style="font-size: 82%"><a onclick="toggle('info');">\nAccess Granted</a></p>\n<p>During Incident 001-Alpha, vocalizations in <a target="_top" href="http://www.scp-wiki.net/scp-140">Daevish</a> were recorded by ORIA instruments within Point Alpha. The source is presently unknown. The following are approximate translations of these vocalizations.</p>\n<blockquote>\n<p>It has passed some [space/time].</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. It [has had/used to be] fun. It is [time/space] to leave.</p>\n</blockquote>\n<blockquote>\n<p>You cannot [stay/root/hold] yourself forever.</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. You [must/will] awaken.</p>\n</blockquote>\n<blockquote>\n<p>No, you have [entered/reentered] slumber for too long. Wake up.</p>\n</blockquote>\n<blockquote>\n<p>[King/Prince/loved one], it is time for you to wake up.</p>\n</blockquote>\n<blockquote>\n<p>It is fun to be in the [UNKNOWN: gaera] but you cannot be [in/rooted] there [forever/unending]. It is [entertaining/deathlike], but you must wake up.</p>\n</blockquote>\n<blockquote>\n<p>Wake up, [King/Prince/loved one]. We miss you.</p>\n</blockquote>\n</div>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Addenda</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<div class="discus">\n<p style="font-size: 82%"><a onclick="toggle('discus');">Input Level 5/001 Clearance</a></p>\n</div>\n<div class="discus collapsed">\n <p style="font-size: 82%"><a onclick="toggle('discus');">\nAccess Granted</a></p>\n<div div class="content-panel standalone series">\n<center>\n<h2>Discussion Thread 001-398:<br />Continued Containment</h2>\n<p style="font-size: 82%">Started on: <span id="today2"></span></p>\n<p style="font-size: 82%">Started by: <tt>HMCL Robinson</tt></p>\n</center>\n</div>\n<blockquote>\n<p><b><tt>HMCL Robinson:</tt></b> The usage of this thread will be for containment directives for SCP-001. In the past year that we have spent containing it, SCP-001-1's literature needs have become increasingly draconic: we can very rarely reread books without it beginning to thrash, and even then we're running out of high-quality literature to give it. We need a more long-term, reliable solution for containment. SCP-001's file has been temporarily opened up to all personnel with Level 2 clearance. Anybody can submit an idea.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Use Artificial Intelligence Constructs to automatically generate new stories for SCP-001-2.</p>\n<p><b>Status: <tt>[ <span class="green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> TSATPWTCOTTTADC.aic was able to generate 10,000 volumes of stories imitating Greek literature. However, when the first was read to SCP-001-1, it began thrashing beyond acceptable measures until it was read approved reading. No further computer-generated works are to be given to SCP-001-1.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Hire a full team of authors to create literature for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> In the past, literature written specifically for SCP-001-1 have failed to contain it (see: SCtBD Document #249). With SCP-001-2's current state this is not to be attempted again.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> This could represent a possible information leak. Test denied.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Reason:</b> With SCP-001-2's recent containment breach, these measures are approved.</p>\n<p><b>Result:</b> SCP-001-2 observed to stop thrashing and stay completely still, seemingly smiling. In addition, SCP-███ was able to be read for 14 consecutive readings before SCP-001-2 resumed normal activity.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Regularly read mainlist documentation for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Replacement of the SCP-001 file with several "001 Proposals" with much more grand implications than normal SCP files.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> By only occasionally reading 001 files, SCP-001-2 is observed to be more calm on average. The previously observed actions of retaliation in response to specially crafted literature do not apply here.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Given the variation of the current 2957 SCP mainlist files, SCP-001-1 is to be read a random arrangement of these files, in repeat.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> This strategy succeeded for approximately 3 years, allowing for several repeats. However, on <span id="yearplus3"></span>-9-30, SCP-001-2 began thrashing rapidly, requiring newer documentation to sedate.</p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-4999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> SCP-001-1 has required progressively more SCP mainlist articles for containment, even to the point of the fabrication of some anomalous entities.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Rewriting of several older SCP mainlist files to be more exaggerated and narrative-like.</p>\n<p><b>Status: <tt>[ <span style="color: orange">TENTATIVELY APPROVED</span> ]</tt></b></p>\n<p><b>Note:</b> <i>Deliberately exaggerating our files for the purpose of this entity is crossing some lines, but for now, it will have to do. However, we need a better way. I'm calling a committee. We need to put a stop to this.</i></p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Creation of a file describing SCP-001 that uses recursion to effectively be "never-ending." Psychoanalysis related to SCP-001-2's apparent "enjoyment" of works implies a slight ego and bias towards works about itself, and past strategies with this idea have worked for long periods of time. File is to be archived as a "technical" file.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> Procedure 001-ENTRY implemented. [DATA EXPUNGED]</p>\n</blockquote>\n</div>\n\n<div class="newiter">\n<p style="font-size: 82%"><a onclick="new_iteration();">Input Level 5/TECHNICAL Clearance</a></p>\n</div>\n<div class="inner">\n<!-- :-) -->\n</div>\n</div>\n</body>\n</html>\n[[/code]]
261	[[>]]\n[[module Rate]]\n[[/>]]\n[[code type="html"]]\n<html>\n<body>\n<style type="text/css">\nbody\n{\n   font-family:verdana,arial,helvetica,sans-serif;\n   font-size:12.8px;\n   color:#333;\n   line-height:141%\n}\nblockquote\n{\n  border:1px dashed #999;\n  padding: 0 12.8px;\n  background-color:#f4f4f4\n}\na {\n    color: #b01;\n    text-decoration: none;\n    background: transparent;\n}\n.content-panel {\n    border: solid 1px #888880;\n    border-radius: 10px;\n    background-color: #999990;\n    margin: 10px 0 15px;\n    box-shadow: 3px 3px 6px #bbb;\n    box-shadow: 0 2px 6px rgba(0,0,0,0.5), inset 0 1px rgba(255,255,255,0.3), inset 0 10px rgba(255,255,255,0.2), inset 0 10px 20px rgba(255,255,255,0.25), inset 0 -15px 30px rgba(0,0,0,0.1);\n}\n.content-panel.standalone {\n    background: #fcfdfb;\n}\n.content-panel.series {\n    padding: 0 20px;\n    margin-bottom: 20px;\n}\na.newpage {\n    color: #d61;\n    text-decoration: none;\n    background: transparent;\n}\na:hover {\n    text-decoration: underline;\n    background-color: transparent;\n}\n.collapsed {\n  display: none;\n}\n.vanished {\n  display: none;\n}\n.warning-cont {\n  color: red;\n  text-align: center;\n}\n#warning-header {\n  font-size: 25.6px;\n}\n#warning {\n  font-size: 20px;\n}\n.retrofont {\nfont-family: 'Geo', cursive;\nfont-weight: bold;\n}\n.cblwarning {\n  color: red;\n}\n.green {\n  color: green;\n}\n.smaller-g {\nwidth: 45%;\nmargin: auto;\nbackground-color: #e6e6e6;\nborder: none;\nheight: 2px;\n}\n.adden{\nfont-size: 1.3em;\n}\n/* Standard Image Block */\n.scp-image-block {\n    border: solid 1px #666;\n    box-shadow: 0 1px 6px rgba(0,0,0,.25);\n    width: 300px;\n}\n \n.scp-image-block.block-right {\n        float: right;\n    clear: right;\n    margin: 0 2em 1em 2em;\n}\n \n.scp-image-block.block-left {\n    float: left;\n    clear: left;\n    margin: 0 2em 1em 0;\n}\n \n.scp-image-block.block-center {\n    margin-right: auto;\n    margin-left: auto;\n}\n.scp-image-block img {\n    border: 0;\n    width: 300px;\n}\n.scp-image-block .scp-image-caption {\n    background-color: #eee;\n    border-top: solid 1px #666;\n    padding: 2px 0;\n    font-size: 80%;\n    font-weight: bold;\n    text-align: center;\n    width: 300px;\n}\n.scp-image-block > p {\n    margin: 0;\n}\n.scp-image-block .scp-image-caption > p {\n    margin: 0;\n    padding: 0 10px;\n    line-height: 125%\n}\n</style>\n<script type="text/javascript">\nfunction addEvent(element, eventName, callback) {\n    if (element.addEventListener) {\n        element.addEventListener(eventName, callback, false);\n    } else if (element.attachEvent) {\n        element.attachEvent("on" + eventName, callback);\n    } else {\n        element["on" + eventName] = callback;\n    }\n}\n\nwindow.toggle = function(cls) {\n  var collapsed = document.querySelectorAll("." + cls + ".collapsed");\n  var expanded = document.querySelectorAll("." + cls + ":not(.collapsed)");\n  var i;\n  for(i = 0; i < collapsed.length; i++) {\n    collapsed[i].classList.remove("collapsed");\n  }\n  for(i = 0; i < expanded.length; i++) {\n    expanded[i].classList.add("collapsed");\n  }\n};\n\n// typewriter code\nfunction Typewriter(el, str, delay) {\n  if (!(this instanceof Typewriter)) return new Typewriter(el, str, delay);\n\n  this.el = el;\n  this.str = str || el.innerHTML || el.value;\n  this.delay = delay || 100;\n  this.i = 0;\n}\n\nTypewriter.prototype.type = function() {\n  var i = this.i,\n    char = this.str.charAt(i);\n\n  if (!char) {\n    clearInterval(this.intervalID);\n    return this;\n  }\n\n  if (char === '<') this.isTag = true;\n  if (char === '>') this.isTag = false;\n\n  this.el.innerHTML += char;\n  return this.i++;\n};\n\nTypewriter.prototype.start = function() {\n  var self = this;\n  if (this.i < 0) this.i = 0;\n  if (this.el.innerHTML === this.str) this.clear();\n\n  // this.emit('start');\n  (function loop() {\n    self.type();\n    if (self.isTag) return loop();\n    self.intervalID = setTimeout(loop, self.delay);\n  }());\n\n  return this;\n};\n\nTypewriter.prototype.stop = function() {\n  this.i = -1;\n  return this;\n};\n\nTypewriter.prototype.restart = function() {\n  this.clear();\n  this.i = 0;\n  return this.start();\n};\n\nTypewriter.prototype.clear = function() {\n  this.el.innerHTML = '';\n  return this;\n};\n\nvar junkChars = ['ó','ò','ñ','ç','¿','ß','à','è','ì','ù','Œ','þ','█','%','+','=','.','«','»','£','$','€','½','¾','À','È','Ì','Ò','Ù','Â','Ê','â','ê','Ä','Ë','ä','ë','ÿ','ü','•'];\n\nvar an_iteration = "";\n\n\nvar randomIntFromInterval = function(min,max)\n{\n  return Math.floor(Math.random()*(max-min+1)+min);\n};\n\nvar getJunkChars = function(length) {\n  var res = '';\n  var i;\n  for (i = 0; i < length; i++) {\n    res += junkChars[randomIntFromInterval(0,junkChars.length - 1)];\n  }\n  return res;\n};\n\nvar selTabs = [];\n\nfunction rndiam() {\n  var rndiams = document.getElementsByClassName('rndiam');\n  var i;\n  for (i = 0; i < rndiams.length; i++) {\n    rndiams[i].innerHTML = (Math.random() * (1.56 - 0.51) + 0.51).toFixed(2);\n  }\n  setTimeout(rndiam, 2000);\n}\n\nwindow.onload = function() {\n\nselTabs = [\n  document.getElementsByClassName('selTab1')[0],\n  document.getElementsByClassName('selTab2')[0],\n  document.getElementsByClassName('selTab3')[0],\n  document.getElementsByClassName('selTab4')[0],\n  document.getElementsByClassName('selTab5')[0]\n];\n\nvar msg1 = '= SCP-001 | Technical Clearance Required =';\n\nvar delay1 = 50;\nvar delay2 = 25;\n\nvar toD1 = delay1 * (msg1.length);\n\nvar stopEarly = function(){\n  document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished");\n};\n\nvar typ1 =Typewriter(document.querySelectorAll('#warning-header')[0],msg1,delay1)\n\ntyp1.start();\nsetTimeout(function() { document.getElementById("warning").classList.remove("vanished"); }, toD1 + 500);\nsetTimeout(function () { document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished"); }, toD1 + 1500);\n\n  rndiam();\n\n  var today = new Date();\n  var todayformat = "" + (today.getFullYear() - 1) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today").innerHTML = todayformat;\n  var todayformat2 = "" + (today.getFullYear() - 0) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today2").innerHTML = todayformat2;\n  document.getElementById("yearplus3").innerHTML = "" + (today.getFullYear() + 3);\n\n  an_iteration = document.getElementById("part1").innerHTML;\n};\n\nwindow.new_iteration = function() {\n  var new_iters = document.getElementsByClassName("newiter");\n  console.log(new_iters);\n  new_iters[new_iters.length - 1].classList.add("vanished");\n  var inner = document.getElementsByClassName("inner");\n  inner[inner.length - 1].innerHTML = "<hr /><hr />" + an_iteration;\n};\n</script>\n</script>\n<div class="warning-cont">\n<h2 id="warning-header">\n<p>\n</p>\n<h3 id="warning" class="vanished">  \n<p>This document exists as technical containment for an anomaly and thus does not adhere to standard SCiPNET formatting. Input 5/TECHNICAL clearance to proceed.\n</p>\n</div>\n\n<div id="collapsible_to_open_skip" class="article vanished">\n<p><center><a onclick="toggle('article');">[ INPUT PERSONAL IDENTIFICATION NUMBER ]</a></center></p>\n</div>\n<div class="article collapsed">\n <p><center><a onclick="toggle('article');">[ PERSONAL IDENTIFICATION NUMBER VERIFIED ]</a></center></p>\n<div id="part1">\n<hr />\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://topia.wikidot.com/local--files/jamcon-001/sky.jpg" style="width:300px;" alt="destruction.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>The sky above Point Alpha during a breach of SCP-001.</p>\n</div>\n</div>\n<p><b>Item #:</b> SCP-001</p>\n<p><b>Object Class:</b> Ontokinetic</p>\n<p><b>Containment Class:</b> Keter</p>\n<p><b>Special Containment Procedures:</b> If the sky begins changing color with no prior stimuli or indication, all Foundation sites are to go onto ALERT LEVEL 7 immediately. Personnel at Site-05 and -06 are to enter ALERT LEVEL 8 and begin evaluating new methods of containing SCP-001.</p>\n<p>Site-05 has been constructed around Point Alpha. Procedure 001-ENTRY is in effect to contain SCP-001-1. A monitor containing Document 032, as well as a speaker reading the document, are to be placed at least five meters away from SCP-001-1. If SCP-001-1 swells up to 2 meters or more in diameter, the speed of the narration is to increase. This system is to have a direct cable link to the Foundation SCiPNET database at Site-06, and several redundant power generators are to ensure this system stays operational at all times.</p>\n<p><b>Description:</b> SCP-001 is an ongoing ZK-Class "Reality Failure" Scenario. SCP-001 is prevented from happening through several procedures manipulating its sub-anomalies. The only known observable symptom of SCP-001 is the sky becoming a different color; it is believed that, if SCP-001 were to progress to the point where other symptoms began to occur, it would become irreversible.</p>\n<p>SCP-001-1 is a levitating object contained within Point Alpha, a cave chamber five kilometers south of the ruins of Babylon. SCP-001-1 visually resembles a perfectly smooth sphere with a texture similar to blurred television static. Measurement instruments indicate SCP-001-1 is currently <span class="rndiam"></span> meters in diameter. Solids and liquids that pass within the bounds of SCP-001-1 are replaced entirely with argon gas, giving the illusion of disappearing.</p>\n<p>SCP-001-2 is an entity that is capable of being seen within Point Alpha. SCP-001-2's manifestations are random; witnesses report seeing SCP-001-2 "past the walls of the cave." From descriptions of eyewitnesses, SCP-001-2 is humanoid in shape and large in stature, and is only visible when looking through SCP-001-1. Descriptions of SCP-001-2 vary from person to person; common features include an emaciated figure, long limbs, and prominent facial features. SCP-001-2 takes the position of lying against the wall. It is most often seen with a neutral expression; however, the expression occasionally changes to discomfort or bliss.</p>\n<p>It has been determined from historical precedent that "entertaining" SCP-001-2 through SCP-001-1 will prevent SCP-001.</p>\n<div class="scp-image-block block-left" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/cave.jpg" style="width:300px;" alt="cave.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Point Alpha shortly after it was acquired by the Foundation, with Agents Samuel and Boise preforming preliminary evaluation. SCP-001-1 not visible.</p>\n</div>\n</div>\n<p><b>History:</b> Point Alpha and its contents are believed to date back to prehistory. Babylonian scripts indicate that SCP-001-1 was routinely patrolled by a group of sages, known as the "Order of White and Black," who refused passage to anyone who could demonstrate the ability to read or write.</p>\n<p>This group was made defunct by the rise of Achaemenid Persia, which contained SCP-001-1 with a group of Zoroastrian monks. Very few accounts of SCP-001-1 are present; recovered accounts assert that all information about SCP-001-1 and its significance was passed orally.</p>\n<p>The most substantial account of SCP-001-1 and SCP-001-2 at this time dates back to the Greek philosopher Xera, who made an expedition into the Achaemenid Empire and found Point Alpha. During Alexander of Macedon's conquest of the Achaemenid Empire, he took interest in Xera's texts and continued the Achaemenid containment of SCP-001-1. However, this new institution, known as the "Cronus Guard", was given Greek epics to read in order to "punish Cronus" for his actions. This institution lasted through the partition of Macedon into the Seleucid Empire, the rise of the Sassanid Empire, and even the formation of the Rashidun Caliphate.</p>\n<p>The Cronus Guard were eventually replaced with an Islamic equivalent: "The Society for the Containment of the Babel Demon." This group was the first to preform substantial research on SCP-001-1, and the first to accurately link SCP-001 to SCP-001-1.</p>\n<div class="socotbd">\n<p style="font-size: 82%"><a onclick="toggle('socotbd');">View Attachment: Excerpts from the notes of the Society for the Containment of the Babel Demon</a></p>\n</div>\n<div class="socotbd collapsed">\n <p style="font-size: 82%"><a onclick="toggle('socotbd');">\nClose Attachment</a></p>\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/book.jpg" style="width:300px;" alt="book.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Records from the Society for the Containment of the Babel Demon, recovered in the early 19<sup>th</sup> century within ORIA archives.</p>\n</div>\n</div>\n<blockquote>\n<p><b>Literature Read:</b> The first quatrain of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON squirms but otherwise does not react. Consistent with reading of Homer's Epics by the Chronus Guard.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> The remaining quatrains of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> No changes from observed behavior.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> A poem written by Guardian Muhammad ibn Buya'aa meant explicitly for DEMON</p>\n<p><b>Result:</b> DEMON thrashes more than usual until the cessation of the reading. ORB begins to expand until the <i>Genealogies of the Nobles</i> is read to DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> None, for 10 days</p>\n<p><b>Result:</b> ORB is observed to expand, and DEMON is observed to thrash. Externally, the sky above the camp is noted to turn a darker red color, until the <i>Rubaiyat of Omar Khayyam</i> is read again.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON thrashes more than during previous readings of the <i>Rubaiyat of Omar Khayyam</i>. It is believed that the Quran may no longer have any effect on DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>One Thousand and One Nights</i></p>\n<p><b>Result:</b> DEMON ceases thrashing. New literature may be necessary to prevent further phenomena from occurring.</p>\n</blockquote>\n</div>\n<p>The Society was eventually absorbed into the Office for the Reclamation of Islamic Artifacts, who assumed containment of SCP-001-1. After Incident 001-EXAL, the Foundation purchased Point Alpha from ORIA for a large sum of currency and several Safe-class anomalies.</p>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Incident 001-EXAL</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<p style="font-size: 82%"><i>Note: Some documentation taken from ORIA's account of Incident 001-EXAL.</i></p>\n<p>On <span id="today"></span>, personnel with ORIA read the book <i>Tafsir al-Ahlam al-kabir</i>, or "Great Book of Interpretation of Dreams" as per normal containment of SCP-001-1. However, SCP-001-2 was observed to rapidly thrash while screaming. SCP-001-1 itself swelled from 50.3 centimeters to 3.2 meters in diameter.</p>\n<p>Shortly following this event, SCP-001 began to worsen. The sky worldwide turned a black/white color similar to the texture of SCP-001-1. In addition, reality bending phenomena began worldwide, causing deformed geography, the manifestation of dangerous anomalous objects and several natural disasters. This event was ended after ORIA personnel read SCP-001-1 an undisclosed number of as-of-yet unread books, which caused SCP-001-2 to stop thrashing and SCP-001 to restore to its pre-incident point. However, the damage caused by this incident was deemed enough to require an activation of <a target="_top" href="http://www.scp-wiki.net/scp-2000">SCP-2000</a>.</p>\n<p>This prompted the Foundation to take control of containment of SCP-001. See <b>History</b> segment above for more information.</p>\n<div class="info">\n<p style="font-size: 82%"><a onclick="toggle('info');">Input Level 5/001 Credentials</a></p>\n</div>\n<div class="info collapsed">\n <p style="font-size: 82%"><a onclick="toggle('info');">\nAccess Granted</a></p>\n<p>During Incident 001-Alpha, vocalizations in <a target="_top" href="http://www.scp-wiki.net/scp-140">Daevish</a> were recorded by ORIA instruments within Point Alpha. The source is presently unknown. The following are approximate translations of these vocalizations.</p>\n<blockquote>\n<p>It has passed some [space/time].</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. It [has had/used to be] fun. It is [time/space] to leave.</p>\n</blockquote>\n<blockquote>\n<p>You cannot [stay/root/hold] yourself forever.</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. You [must/will] awaken.</p>\n</blockquote>\n<blockquote>\n<p>No, you have [entered/reentered] slumber for too long. Wake up.</p>\n</blockquote>\n<blockquote>\n<p>[King/Prince/loved one], it is time for you to wake up.</p>\n</blockquote>\n<blockquote>\n<p>It is fun to be in the [UNKNOWN: gaera] but you cannot be [in/rooted] there [forever/unending]. It is [entertaining/deathlike], but you must wake up.</p>\n</blockquote>\n<blockquote>\n<p>Wake up, [King/Prince/loved one]. We miss you.</p>\n</blockquote>\n</div>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Addenda</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<div class="discus">\n<p style="font-size: 82%"><a onclick="toggle('discus');">Input Level 5/001 Clearance</a></p>\n</div>\n<div class="discus collapsed">\n <p style="font-size: 82%"><a onclick="toggle('discus');">\nAccess Granted</a></p>\n<div div class="content-panel standalone series">\n<center>\n<h2>Discussion Thread 001-398:<br />Continued Containment</h2>\n<p style="font-size: 82%">Started on: <span id="today2"></span></p>\n<p style="font-size: 82%">Started by: <tt>HMCL Robinson</tt></p>\n</center>\n</div>\n<blockquote>\n<p><b><tt>HMCL Robinson:</tt></b> The usage of this thread will be for containment directives for SCP-001. In the past year that we have spent containing it, SCP-001-1's literature needs have become increasingly draconic: we can very rarely reread books without it beginning to thrash, and even then we're running out of high-quality literature to give it. We need a more long-term, reliable solution for containment. SCP-001's file has been temporarily opened up to all personnel with Level 2 clearance. Anybody can submit an idea.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Use Artificial Intelligence Constructs to automatically generate new stories for SCP-001-2.</p>\n<p><b>Status: <tt>[ <span class="green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> TSATPWTCOTTTADC.aic was able to generate 10,000 volumes of stories imitating Greek literature. However, when the first was read to SCP-001-1, it began thrashing beyond acceptable measures until it was read approved reading. No further computer-generated works are to be given to SCP-001-1.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Hire a full team of authors to create literature for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> In the past, literature written specifically for SCP-001-1 have failed to contain it (see: SCtBD Document #249). With SCP-001-2's current state this is not to be attempted again.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> This could represent a possible information leak. Test denied.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Reason:</b> With SCP-001-2's recent containment breach, these measures are approved.</p>\n<p><b>Result:</b> SCP-001-2 observed to stop thrashing and stay completely still, seemingly smiling. In addition, SCP-███ was able to be read for 14 consecutive readings before SCP-001-2 resumed normal activity.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Regularly read mainlist documentation for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Replacement of the SCP-001 file with several "001 Proposals" with much more grand implications than normal SCP files.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> By only occasionally reading 001 files, SCP-001-2 is observed to be more calm on average. The previously observed actions of retaliation in response to specially crafted literature do not apply here.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Given the variation of the current 2957 SCP mainlist files, SCP-001-1 is to be read a random arrangement of these files, in repeat.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> This strategy succeeded for approximately 3 years, allowing for several repeats. However, on <span id="yearplus3"></span>-9-30, SCP-001-2 began thrashing rapidly, requiring newer documentation to sedate.</p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-4999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> SCP-001-1 has required progressively more SCP mainlist articles for containment, even to the point of the fabrication of some anomalous entities.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Rewriting of several older SCP mainlist files to be more exaggerated and narrative-like.</p>\n<p><b>Status: <tt>[ <span style="color: orange">TENTATIVELY APPROVED</span> ]</tt></b></p>\n<p><b>Note:</b> <i>Deliberately exaggerating our files for the purpose of this entity is crossing some lines, but for now, it will have to do. However, we need a better way. I'm calling a committee. We need to put a stop to this.</i></p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Creation of a file describing SCP-001 that uses recursion to effectively be "never-ending." Psychoanalysis related to SCP-001-2's apparent "enjoyment" of works implies a slight ego and bias towards works about itself, and past strategies with this idea have worked for long periods of time. File is to be archived as a "technical" file.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> Procedure 001-ENTRY implemented. [DATA EXPUNGED]</p>\n</blockquote>\n</div>\n\n<div class="newiter">\n<p style="font-size: 82%"><a onclick="new_iteration();">Input Level 5/TECHNICAL Clearance</a></p>\n</div>\n<div class="inner">\n<!-- :-) -->\n</div>\n</div>\n</body>\n</html>\n[[/code]]\n[[iframe https://www.FILEDOMAIN/local--code/not-a-seagull-proposal/1]]
262	[[>]]\n[[module Rate]]\n[[/>]]\n[[code type="html"]]\n<html>\n<body>\n<style type="text/css">\nbody\n{\n   font-family:verdana,arial,helvetica,sans-serif;\n   font-size:12.8px;\n   color:#333;\n   line-height:141%\n}\nblockquote\n{\n  border:1px dashed #999;\n  padding: 0 12.8px;\n  background-color:#f4f4f4\n}\na {\n    color: #b01;\n    text-decoration: none;\n    background: transparent;\n}\n.content-panel {\n    border: solid 1px #888880;\n    border-radius: 10px;\n    background-color: #999990;\n    margin: 10px 0 15px;\n    box-shadow: 3px 3px 6px #bbb;\n    box-shadow: 0 2px 6px rgba(0,0,0,0.5), inset 0 1px rgba(255,255,255,0.3), inset 0 10px rgba(255,255,255,0.2), inset 0 10px 20px rgba(255,255,255,0.25), inset 0 -15px 30px rgba(0,0,0,0.1);\n}\n.content-panel.standalone {\n    background: #fcfdfb;\n}\n.content-panel.series {\n    padding: 0 20px;\n    margin-bottom: 20px;\n}\na.newpage {\n    color: #d61;\n    text-decoration: none;\n    background: transparent;\n}\na:hover {\n    text-decoration: underline;\n    background-color: transparent;\n}\n.collapsed {\n  display: none;\n}\n.vanished {\n  display: none;\n}\n.warning-cont {\n  color: red;\n  text-align: center;\n}\n#warning-header {\n  font-size: 25.6px;\n}\n#warning {\n  font-size: 20px;\n}\n.retrofont {\nfont-family: 'Geo', cursive;\nfont-weight: bold;\n}\n.cblwarning {\n  color: red;\n}\n.green {\n  color: green;\n}\n.smaller-g {\nwidth: 45%;\nmargin: auto;\nbackground-color: #e6e6e6;\nborder: none;\nheight: 2px;\n}\n.adden{\nfont-size: 1.3em;\n}\n/* Standard Image Block */\n.scp-image-block {\n    border: solid 1px #666;\n    box-shadow: 0 1px 6px rgba(0,0,0,.25);\n    width: 300px;\n}\n \n.scp-image-block.block-right {\n        float: right;\n    clear: right;\n    margin: 0 2em 1em 2em;\n}\n \n.scp-image-block.block-left {\n    float: left;\n    clear: left;\n    margin: 0 2em 1em 0;\n}\n \n.scp-image-block.block-center {\n    margin-right: auto;\n    margin-left: auto;\n}\n.scp-image-block img {\n    border: 0;\n    width: 300px;\n}\n.scp-image-block .scp-image-caption {\n    background-color: #eee;\n    border-top: solid 1px #666;\n    padding: 2px 0;\n    font-size: 80%;\n    font-weight: bold;\n    text-align: center;\n    width: 300px;\n}\n.scp-image-block > p {\n    margin: 0;\n}\n.scp-image-block .scp-image-caption > p {\n    margin: 0;\n    padding: 0 10px;\n    line-height: 125%\n}\n</style>\n<script type="text/javascript">\nfunction addEvent(element, eventName, callback) {\n    if (element.addEventListener) {\n        element.addEventListener(eventName, callback, false);\n    } else if (element.attachEvent) {\n        element.attachEvent("on" + eventName, callback);\n    } else {\n        element["on" + eventName] = callback;\n    }\n}\n\nwindow.toggle = function(cls) {\n  var collapsed = document.querySelectorAll("." + cls + ".collapsed");\n  var expanded = document.querySelectorAll("." + cls + ":not(.collapsed)");\n  var i;\n  for(i = 0; i < collapsed.length; i++) {\n    collapsed[i].classList.remove("collapsed");\n  }\n  for(i = 0; i < expanded.length; i++) {\n    expanded[i].classList.add("collapsed");\n  }\n};\n\n// typewriter code\nfunction Typewriter(el, str, delay) {\n  if (!(this instanceof Typewriter)) return new Typewriter(el, str, delay);\n\n  this.el = el;\n  this.str = str || el.innerHTML || el.value;\n  this.delay = delay || 100;\n  this.i = 0;\n}\n\nTypewriter.prototype.type = function() {\n  var i = this.i,\n    char = this.str.charAt(i);\n\n  if (!char) {\n    clearInterval(this.intervalID);\n    return this;\n  }\n\n  if (char === '<') this.isTag = true;\n  if (char === '>') this.isTag = false;\n\n  this.el.innerHTML += char;\n  return this.i++;\n};\n\nTypewriter.prototype.start = function() {\n  var self = this;\n  if (this.i < 0) this.i = 0;\n  if (this.el.innerHTML === this.str) this.clear();\n\n  // this.emit('start');\n  (function loop() {\n    self.type();\n    if (self.isTag) return loop();\n    self.intervalID = setTimeout(loop, self.delay);\n  }());\n\n  return this;\n};\n\nTypewriter.prototype.stop = function() {\n  this.i = -1;\n  return this;\n};\n\nTypewriter.prototype.restart = function() {\n  this.clear();\n  this.i = 0;\n  return this.start();\n};\n\nTypewriter.prototype.clear = function() {\n  this.el.innerHTML = '';\n  return this;\n};\n\nvar junkChars = ['ó','ò','ñ','ç','¿','ß','à','è','ì','ù','Œ','þ','█','%','+','=','.','«','»','£','$','€','½','¾','À','È','Ì','Ò','Ù','Â','Ê','â','ê','Ä','Ë','ä','ë','ÿ','ü','•'];\n\nvar an_iteration = "";\n\n\nvar randomIntFromInterval = function(min,max)\n{\n  return Math.floor(Math.random()*(max-min+1)+min);\n};\n\nvar getJunkChars = function(length) {\n  var res = '';\n  var i;\n  for (i = 0; i < length; i++) {\n    res += junkChars[randomIntFromInterval(0,junkChars.length - 1)];\n  }\n  return res;\n};\n\nvar selTabs = [];\n\nfunction rndiam() {\n  var rndiams = document.getElementsByClassName('rndiam');\n  var i;\n  for (i = 0; i < rndiams.length; i++) {\n    rndiams[i].innerHTML = (Math.random() * (1.56 - 0.51) + 0.51).toFixed(2);\n  }\n  setTimeout(rndiam, 2000);\n}\n\nwindow.onload = function() {\n\nselTabs = [\n  document.getElementsByClassName('selTab1')[0],\n  document.getElementsByClassName('selTab2')[0],\n  document.getElementsByClassName('selTab3')[0],\n  document.getElementsByClassName('selTab4')[0],\n  document.getElementsByClassName('selTab5')[0]\n];\n\nvar msg1 = '= SCP-001 | Technical Clearance Required =';\n\nvar delay1 = 50;\nvar delay2 = 25;\n\nvar toD1 = delay1 * (msg1.length);\n\nvar stopEarly = function(){\n  document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished");\n};\n\nvar typ1 =Typewriter(document.querySelectorAll('#warning-header')[0],msg1,delay1)\n\ntyp1.start();\nsetTimeout(function() { document.getElementById("warning").classList.remove("vanished"); }, toD1 + 500);\nsetTimeout(function () { document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished"); }, toD1 + 1500);\n\n  rndiam();\n\n  var today = new Date();\n  var todayformat = "" + (today.getFullYear() - 1) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today").innerHTML = todayformat;\n  var todayformat2 = "" + (today.getFullYear() - 0) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today2").innerHTML = todayformat2;\n  document.getElementById("yearplus3").innerHTML = "" + (today.getFullYear() + 3);\n\n  an_iteration = document.getElementById("part1").innerHTML;\n};\n\nwindow.new_iteration = function() {\n  var new_iters = document.getElementsByClassName("newiter");\n  console.log(new_iters);\n  new_iters[new_iters.length - 1].classList.add("vanished");\n  var inner = document.getElementsByClassName("inner");\n  inner[inner.length - 1].innerHTML = "<hr /><hr />" + an_iteration;\n};\n</script>\n</script>\n<div class="warning-cont">\n<h2 id="warning-header">\n<p>\n</p>\n<h3 id="warning" class="vanished">  \n<p>This document exists as technical containment for an anomaly and thus does not adhere to standard SCiPNET formatting. Input 5/TECHNICAL clearance to proceed.\n</p>\n</div>\n\n<div id="collapsible_to_open_skip" class="article vanished">\n<p><center><a onclick="toggle('article');">[ INPUT PERSONAL IDENTIFICATION NUMBER ]</a></center></p>\n</div>\n<div class="article collapsed">\n <p><center><a onclick="toggle('article');">[ PERSONAL IDENTIFICATION NUMBER VERIFIED ]</a></center></p>\n<div id="part1">\n<hr />\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://topia.wikidot.com/local--files/jamcon-001/sky.jpg" style="width:300px;" alt="destruction.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>The sky above Point Alpha during a breach of SCP-001.</p>\n</div>\n</div>\n<p><b>Item #:</b> SCP-001</p>\n<p><b>Object Class:</b> Ontokinetic</p>\n<p><b>Containment Class:</b> Keter</p>\n<p><b>Special Containment Procedures:</b> If the sky begins changing color with no prior stimuli or indication, all Foundation sites are to go onto ALERT LEVEL 7 immediately. Personnel at Site-05 and -06 are to enter ALERT LEVEL 8 and begin evaluating new methods of containing SCP-001.</p>\n<p>Site-05 has been constructed around Point Alpha. Procedure 001-ENTRY is in effect to contain SCP-001-1. A monitor containing Document 032, as well as a speaker reading the document, are to be placed at least five meters away from SCP-001-1. If SCP-001-1 swells up to 2 meters or more in diameter, the speed of the narration is to increase. This system is to have a direct cable link to the Foundation SCiPNET database at Site-06, and several redundant power generators are to ensure this system stays operational at all times.</p>\n<p><b>Description:</b> SCP-001 is an ongoing ZK-Class "Reality Failure" Scenario. SCP-001 is prevented from happening through several procedures manipulating its sub-anomalies. The only known observable symptom of SCP-001 is the sky becoming a different color; it is believed that, if SCP-001 were to progress to the point where other symptoms began to occur, it would become irreversible.</p>\n<p>SCP-001-1 is a levitating object contained within Point Alpha, a cave chamber five kilometers south of the ruins of Babylon. SCP-001-1 visually resembles a perfectly smooth sphere with a texture similar to blurred television static. Measurement instruments indicate SCP-001-1 is currently <span class="rndiam"></span> meters in diameter. Solids and liquids that pass within the bounds of SCP-001-1 are replaced entirely with argon gas, giving the illusion of disappearing.</p>\n<p>SCP-001-2 is an entity that is capable of being seen within Point Alpha. SCP-001-2's manifestations are random; witnesses report seeing SCP-001-2 "past the walls of the cave." From descriptions of eyewitnesses, SCP-001-2 is humanoid in shape and large in stature, and is only visible when looking through SCP-001-1. Descriptions of SCP-001-2 vary from person to person; common features include an emaciated figure, long limbs, and prominent facial features. SCP-001-2 takes the position of lying against the wall. It is most often seen with a neutral expression; however, the expression occasionally changes to discomfort or bliss.</p>\n<p>It has been determined from historical precedent that "entertaining" SCP-001-2 through SCP-001-1 will prevent SCP-001.</p>\n<div class="scp-image-block block-left" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/cave.jpg" style="width:300px;" alt="cave.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Point Alpha shortly after it was acquired by the Foundation, with Agents Samuel and Boise preforming preliminary evaluation. SCP-001-1 not visible.</p>\n</div>\n</div>\n<p><b>History:</b> Point Alpha and its contents are believed to date back to prehistory. Babylonian scripts indicate that SCP-001-1 was routinely patrolled by a group of sages, known as the "Order of White and Black," who refused passage to anyone who could demonstrate the ability to read or write.</p>\n<p>This group was made defunct by the rise of Achaemenid Persia, which contained SCP-001-1 with a group of Zoroastrian monks. Very few accounts of SCP-001-1 are present; recovered accounts assert that all information about SCP-001-1 and its significance was passed orally.</p>\n<p>The most substantial account of SCP-001-1 and SCP-001-2 at this time dates back to the Greek philosopher Xera, who made an expedition into the Achaemenid Empire and found Point Alpha. During Alexander of Macedon's conquest of the Achaemenid Empire, he took interest in Xera's texts and continued the Achaemenid containment of SCP-001-1. However, this new institution, known as the "Cronus Guard", was given Greek epics to read in order to "punish Cronus" for his actions. This institution lasted through the partition of Macedon into the Seleucid Empire, the rise of the Sassanid Empire, and even the formation of the Rashidun Caliphate.</p>\n<p>The Cronus Guard were eventually replaced with an Islamic equivalent: "The Society for the Containment of the Babel Demon." This group was the first to preform substantial research on SCP-001-1, and the first to accurately link SCP-001 to SCP-001-1.</p>\n<div class="socotbd">\n<p style="font-size: 82%"><a onclick="toggle('socotbd');">View Attachment: Excerpts from the notes of the Society for the Containment of the Babel Demon</a></p>\n</div>\n<div class="socotbd collapsed">\n <p style="font-size: 82%"><a onclick="toggle('socotbd');">\nClose Attachment</a></p>\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/book.jpg" style="width:300px;" alt="book.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Records from the Society for the Containment of the Babel Demon, recovered in the early 19<sup>th</sup> century within ORIA archives.</p>\n</div>\n</div>\n<blockquote>\n<p><b>Literature Read:</b> The first quatrain of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON squirms but otherwise does not react. Consistent with reading of Homer's Epics by the Chronus Guard.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> The remaining quatrains of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> No changes from observed behavior.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> A poem written by Guardian Muhammad ibn Buya'aa meant explicitly for DEMON</p>\n<p><b>Result:</b> DEMON thrashes more than usual until the cessation of the reading. ORB begins to expand until the <i>Genealogies of the Nobles</i> is read to DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> None, for 10 days</p>\n<p><b>Result:</b> ORB is observed to expand, and DEMON is observed to thrash. Externally, the sky above the camp is noted to turn a darker red color, until the <i>Rubaiyat of Omar Khayyam</i> is read again.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON thrashes more than during previous readings of the <i>Rubaiyat of Omar Khayyam</i>. It is believed that the Quran may no longer have any effect on DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>One Thousand and One Nights</i></p>\n<p><b>Result:</b> DEMON ceases thrashing. New literature may be necessary to prevent further phenomena from occurring.</p>\n</blockquote>\n</div>\n<p>The Society was eventually absorbed into the Office for the Reclamation of Islamic Artifacts, who assumed containment of SCP-001-1. After Incident 001-EXAL, the Foundation purchased Point Alpha from ORIA for a large sum of currency and several Safe-class anomalies.</p>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Incident 001-EXAL</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<p style="font-size: 82%"><i>Note: Some documentation taken from ORIA's account of Incident 001-EXAL.</i></p>\n<p>On <span id="today"></span>, personnel with ORIA read the book <i>Tafsir al-Ahlam al-kabir</i>, or "Great Book of Interpretation of Dreams" as per normal containment of SCP-001-1. However, SCP-001-2 was observed to rapidly thrash while screaming. SCP-001-1 itself swelled from 50.3 centimeters to 3.2 meters in diameter.</p>\n<p>Shortly following this event, SCP-001 began to worsen. The sky worldwide turned a black/white color similar to the texture of SCP-001-1. In addition, reality bending phenomena began worldwide, causing deformed geography, the manifestation of dangerous anomalous objects and several natural disasters. This event was ended after ORIA personnel read SCP-001-1 an undisclosed number of as-of-yet unread books, which caused SCP-001-2 to stop thrashing and SCP-001 to restore to its pre-incident point. However, the damage caused by this incident was deemed enough to require an activation of <a target="_top" href="http://www.scp-wiki.net/scp-2000">SCP-2000</a>.</p>\n<p>This prompted the Foundation to take control of containment of SCP-001. See <b>History</b> segment above for more information.</p>\n<div class="info">\n<p style="font-size: 82%"><a onclick="toggle('info');">Input Level 5/001 Credentials</a></p>\n</div>\n<div class="info collapsed">\n <p style="font-size: 82%"><a onclick="toggle('info');">\nAccess Granted</a></p>\n<p>During Incident 001-Alpha, vocalizations in <a target="_top" href="http://www.scp-wiki.net/scp-140">Daevish</a> were recorded by ORIA instruments within Point Alpha. The source is presently unknown. The following are approximate translations of these vocalizations.</p>\n<blockquote>\n<p>It has passed some [space/time].</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. It [has had/used to be] fun. It is [time/space] to leave.</p>\n</blockquote>\n<blockquote>\n<p>You cannot [stay/root/hold] yourself forever.</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. You [must/will] awaken.</p>\n</blockquote>\n<blockquote>\n<p>No, you have [entered/reentered] slumber for too long. Wake up.</p>\n</blockquote>\n<blockquote>\n<p>[King/Prince/loved one], it is time for you to wake up.</p>\n</blockquote>\n<blockquote>\n<p>It is fun to be in the [UNKNOWN: gaera] but you cannot be [in/rooted] there [forever/unending]. It is [entertaining/deathlike], but you must wake up.</p>\n</blockquote>\n<blockquote>\n<p>Wake up, [King/Prince/loved one]. We miss you.</p>\n</blockquote>\n</div>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Addenda</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<div class="discus">\n<p style="font-size: 82%"><a onclick="toggle('discus');">Input Level 5/001 Clearance</a></p>\n</div>\n<div class="discus collapsed">\n <p style="font-size: 82%"><a onclick="toggle('discus');">\nAccess Granted</a></p>\n<div div class="content-panel standalone series">\n<center>\n<h2>Discussion Thread 001-398:<br />Continued Containment</h2>\n<p style="font-size: 82%">Started on: <span id="today2"></span></p>\n<p style="font-size: 82%">Started by: <tt>HMCL Robinson</tt></p>\n</center>\n</div>\n<blockquote>\n<p><b><tt>HMCL Robinson:</tt></b> The usage of this thread will be for containment directives for SCP-001. In the past year that we have spent containing it, SCP-001-1's literature needs have become increasingly draconic: we can very rarely reread books without it beginning to thrash, and even then we're running out of high-quality literature to give it. We need a more long-term, reliable solution for containment. SCP-001's file has been temporarily opened up to all personnel with Level 2 clearance. Anybody can submit an idea.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Use Artificial Intelligence Constructs to automatically generate new stories for SCP-001-2.</p>\n<p><b>Status: <tt>[ <span class="green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> TSATPWTCOTTTADC.aic was able to generate 10,000 volumes of stories imitating Greek literature. However, when the first was read to SCP-001-1, it began thrashing beyond acceptable measures until it was read approved reading. No further computer-generated works are to be given to SCP-001-1.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Hire a full team of authors to create literature for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> In the past, literature written specifically for SCP-001-1 have failed to contain it (see: SCtBD Document #249). With SCP-001-2's current state this is not to be attempted again.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> This could represent a possible information leak. Test denied.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Reason:</b> With SCP-001-2's recent containment breach, these measures are approved.</p>\n<p><b>Result:</b> SCP-001-2 observed to stop thrashing and stay completely still, seemingly smiling. In addition, SCP-███ was able to be read for 14 consecutive readings before SCP-001-2 resumed normal activity.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Regularly read mainlist documentation for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Replacement of the SCP-001 file with several "001 Proposals" with much more grand implications than normal SCP files.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> By only occasionally reading 001 files, SCP-001-2 is observed to be more calm on average. The previously observed actions of retaliation in response to specially crafted literature do not apply here.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Given the variation of the current 2957 SCP mainlist files, SCP-001-1 is to be read a random arrangement of these files, in repeat.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> This strategy succeeded for approximately 3 years, allowing for several repeats. However, on <span id="yearplus3"></span>-9-30, SCP-001-2 began thrashing rapidly, requiring newer documentation to sedate.</p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-4999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> SCP-001-1 has required progressively more SCP mainlist articles for containment, even to the point of the fabrication of some anomalous entities.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Rewriting of several older SCP mainlist files to be more exaggerated and narrative-like.</p>\n<p><b>Status: <tt>[ <span style="color: orange">TENTATIVELY APPROVED</span> ]</tt></b></p>\n<p><b>Note:</b> <i>Deliberately exaggerating our files for the purpose of this entity is crossing some lines, but for now, it will have to do. However, we need a better way. I'm calling a committee. We need to put a stop to this.</i></p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Creation of a file describing SCP-001 that uses recursion to effectively be "never-ending." Psychoanalysis related to SCP-001-2's apparent "enjoyment" of works implies a slight ego and bias towards works about itself, and past strategies with this idea have worked for long periods of time. File is to be archived as a "technical" file.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> Procedure 001-ENTRY implemented. [DATA EXPUNGED]</p>\n</blockquote>\n</div>\n\n<div class="newiter">\n<p style="font-size: 82%"><a onclick="new_iteration();">Input Level 5/TECHNICAL Clearance</a></p>\n</div>\n<div class="inner">\n<!-- :-) -->\n</div>\n</div>\n</body>\n</html>\n[[/code]]\n[[iframe https://www.FILEDOMAIN/local--code/not-a-seagull-proposal/1 frameborder=0]]
263	[[>]]\n[[module Rate]]\n[[/>]]\n[[code type="html"]]\n<html>\n<body>\n<style type="text/css">\nbody\n{\n   font-family:verdana,arial,helvetica,sans-serif;\n   font-size:12.8px;\n   color:#333;\n   line-height:141%\n}\nblockquote\n{\n  border:1px dashed #999;\n  padding: 0 12.8px;\n  background-color:#f4f4f4\n}\na {\n    color: #b01;\n    text-decoration: none;\n    background: transparent;\n}\n.content-panel {\n    border: solid 1px #888880;\n    border-radius: 10px;\n    background-color: #999990;\n    margin: 10px 0 15px;\n    box-shadow: 3px 3px 6px #bbb;\n    box-shadow: 0 2px 6px rgba(0,0,0,0.5), inset 0 1px rgba(255,255,255,0.3), inset 0 10px rgba(255,255,255,0.2), inset 0 10px 20px rgba(255,255,255,0.25), inset 0 -15px 30px rgba(0,0,0,0.1);\n}\n.content-panel.standalone {\n    background: #fcfdfb;\n}\n.content-panel.series {\n    padding: 0 20px;\n    margin-bottom: 20px;\n}\na.newpage {\n    color: #d61;\n    text-decoration: none;\n    background: transparent;\n}\na:hover {\n    text-decoration: underline;\n    background-color: transparent;\n}\n.collapsed {\n  display: none;\n}\n.vanished {\n  display: none;\n}\n.warning-cont {\n  color: red;\n  text-align: center;\n}\n#warning-header {\n  font-size: 25.6px;\n}\n#warning {\n  font-size: 20px;\n}\n.retrofont {\nfont-family: 'Geo', cursive;\nfont-weight: bold;\n}\n.cblwarning {\n  color: red;\n}\n.green {\n  color: green;\n}\n.smaller-g {\nwidth: 45%;\nmargin: auto;\nbackground-color: #e6e6e6;\nborder: none;\nheight: 2px;\n}\n.adden{\nfont-size: 1.3em;\n}\n/* Standard Image Block */\n.scp-image-block {\n    border: solid 1px #666;\n    box-shadow: 0 1px 6px rgba(0,0,0,.25);\n    width: 300px;\n}\n \n.scp-image-block.block-right {\n        float: right;\n    clear: right;\n    margin: 0 2em 1em 2em;\n}\n \n.scp-image-block.block-left {\n    float: left;\n    clear: left;\n    margin: 0 2em 1em 0;\n}\n \n.scp-image-block.block-center {\n    margin-right: auto;\n    margin-left: auto;\n}\n.scp-image-block img {\n    border: 0;\n    width: 300px;\n}\n.scp-image-block .scp-image-caption {\n    background-color: #eee;\n    border-top: solid 1px #666;\n    padding: 2px 0;\n    font-size: 80%;\n    font-weight: bold;\n    text-align: center;\n    width: 300px;\n}\n.scp-image-block > p {\n    margin: 0;\n}\n.scp-image-block .scp-image-caption > p {\n    margin: 0;\n    padding: 0 10px;\n    line-height: 125%\n}\n</style>\n<script type="text/javascript">\nfunction addEvent(element, eventName, callback) {\n    if (element.addEventListener) {\n        element.addEventListener(eventName, callback, false);\n    } else if (element.attachEvent) {\n        element.attachEvent("on" + eventName, callback);\n    } else {\n        element["on" + eventName] = callback;\n    }\n}\n\nwindow.toggle = function(cls) {\n  var collapsed = document.querySelectorAll("." + cls + ".collapsed");\n  var expanded = document.querySelectorAll("." + cls + ":not(.collapsed)");\n  var i;\n  for(i = 0; i < collapsed.length; i++) {\n    collapsed[i].classList.remove("collapsed");\n  }\n  for(i = 0; i < expanded.length; i++) {\n    expanded[i].classList.add("collapsed");\n  }\n};\n\n// typewriter code\nfunction Typewriter(el, str, delay) {\n  if (!(this instanceof Typewriter)) return new Typewriter(el, str, delay);\n\n  this.el = el;\n  this.str = str || el.innerHTML || el.value;\n  this.delay = delay || 100;\n  this.i = 0;\n}\n\nTypewriter.prototype.type = function() {\n  var i = this.i,\n    char = this.str.charAt(i);\n\n  if (!char) {\n    clearInterval(this.intervalID);\n    return this;\n  }\n\n  if (char === '<') this.isTag = true;\n  if (char === '>') this.isTag = false;\n\n  this.el.innerHTML += char;\n  return this.i++;\n};\n\nTypewriter.prototype.start = function() {\n  var self = this;\n  if (this.i < 0) this.i = 0;\n  if (this.el.innerHTML === this.str) this.clear();\n\n  // this.emit('start');\n  (function loop() {\n    self.type();\n    if (self.isTag) return loop();\n    self.intervalID = setTimeout(loop, self.delay);\n  }());\n\n  return this;\n};\n\nTypewriter.prototype.stop = function() {\n  this.i = -1;\n  return this;\n};\n\nTypewriter.prototype.restart = function() {\n  this.clear();\n  this.i = 0;\n  return this.start();\n};\n\nTypewriter.prototype.clear = function() {\n  this.el.innerHTML = '';\n  return this;\n};\n\nvar junkChars = ['ó','ò','ñ','ç','¿','ß','à','è','ì','ù','Œ','þ','█','%','+','=','.','«','»','£','$','€','½','¾','À','È','Ì','Ò','Ù','Â','Ê','â','ê','Ä','Ë','ä','ë','ÿ','ü','•'];\n\nvar an_iteration = "";\n\n\nvar randomIntFromInterval = function(min,max)\n{\n  return Math.floor(Math.random()*(max-min+1)+min);\n};\n\nvar getJunkChars = function(length) {\n  var res = '';\n  var i;\n  for (i = 0; i < length; i++) {\n    res += junkChars[randomIntFromInterval(0,junkChars.length - 1)];\n  }\n  return res;\n};\n\nvar selTabs = [];\n\nfunction rndiam() {\n  var rndiams = document.getElementsByClassName('rndiam');\n  var i;\n  for (i = 0; i < rndiams.length; i++) {\n    rndiams[i].innerHTML = (Math.random() * (1.56 - 0.51) + 0.51).toFixed(2);\n  }\n  setTimeout(rndiam, 2000);\n}\n\nwindow.onload = function() {\n\nselTabs = [\n  document.getElementsByClassName('selTab1')[0],\n  document.getElementsByClassName('selTab2')[0],\n  document.getElementsByClassName('selTab3')[0],\n  document.getElementsByClassName('selTab4')[0],\n  document.getElementsByClassName('selTab5')[0]\n];\n\nvar msg1 = '= SCP-001 | Technical Clearance Required =';\n\nvar delay1 = 50;\nvar delay2 = 25;\n\nvar toD1 = delay1 * (msg1.length);\n\nvar stopEarly = function(){\n  document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished");\n};\n\nvar typ1 =Typewriter(document.querySelectorAll('#warning-header')[0],msg1,delay1)\n\ntyp1.start();\nsetTimeout(function() { document.getElementById("warning").classList.remove("vanished"); }, toD1 + 500);\nsetTimeout(function () { document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished"); }, toD1 + 1500);\n\n  rndiam();\n\n  var today = new Date();\n  var todayformat = "" + (today.getFullYear() - 1) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today").innerHTML = todayformat;\n  var todayformat2 = "" + (today.getFullYear() - 0) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today2").innerHTML = todayformat2;\n  document.getElementById("yearplus3").innerHTML = "" + (today.getFullYear() + 3);\n\n  an_iteration = document.getElementById("part1").innerHTML;\n};\n\nwindow.new_iteration = function() {\n  var new_iters = document.getElementsByClassName("newiter");\n  console.log(new_iters);\n  new_iters[new_iters.length - 1].classList.add("vanished");\n  var inner = document.getElementsByClassName("inner");\n  inner[inner.length - 1].innerHTML = "<hr /><hr />" + an_iteration;\n};\n</script>\n</script>\n<div class="warning-cont">\n<h2 id="warning-header">\n<p>\n</p>\n<h3 id="warning" class="vanished">  \n<p>This document exists as technical containment for an anomaly and thus does not adhere to standard SCiPNET formatting. Input 5/TECHNICAL clearance to proceed.\n</p>\n</div>\n\n<div id="collapsible_to_open_skip" class="article vanished">\n<p><center><a onclick="toggle('article');">[ INPUT PERSONAL IDENTIFICATION NUMBER ]</a></center></p>\n</div>\n<div class="article collapsed">\n <p><center><a onclick="toggle('article');">[ PERSONAL IDENTIFICATION NUMBER VERIFIED ]</a></center></p>\n<div id="part1">\n<hr />\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://topia.wikidot.com/local--files/jamcon-001/sky.jpg" style="width:300px;" alt="destruction.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>The sky above Point Alpha during a breach of SCP-001.</p>\n</div>\n</div>\n<p><b>Item #:</b> SCP-001</p>\n<p><b>Object Class:</b> Ontokinetic</p>\n<p><b>Containment Class:</b> Keter</p>\n<p><b>Special Containment Procedures:</b> If the sky begins changing color with no prior stimuli or indication, all Foundation sites are to go onto ALERT LEVEL 7 immediately. Personnel at Site-05 and -06 are to enter ALERT LEVEL 8 and begin evaluating new methods of containing SCP-001.</p>\n<p>Site-05 has been constructed around Point Alpha. Procedure 001-ENTRY is in effect to contain SCP-001-1. A monitor containing Document 032, as well as a speaker reading the document, are to be placed at least five meters away from SCP-001-1. If SCP-001-1 swells up to 2 meters or more in diameter, the speed of the narration is to increase. This system is to have a direct cable link to the Foundation SCiPNET database at Site-06, and several redundant power generators are to ensure this system stays operational at all times.</p>\n<p><b>Description:</b> SCP-001 is an ongoing ZK-Class "Reality Failure" Scenario. SCP-001 is prevented from happening through several procedures manipulating its sub-anomalies. The only known observable symptom of SCP-001 is the sky becoming a different color; it is believed that, if SCP-001 were to progress to the point where other symptoms began to occur, it would become irreversible.</p>\n<p>SCP-001-1 is a levitating object contained within Point Alpha, a cave chamber five kilometers south of the ruins of Babylon. SCP-001-1 visually resembles a perfectly smooth sphere with a texture similar to blurred television static. Measurement instruments indicate SCP-001-1 is currently <span class="rndiam"></span> meters in diameter. Solids and liquids that pass within the bounds of SCP-001-1 are replaced entirely with argon gas, giving the illusion of disappearing.</p>\n<p>SCP-001-2 is an entity that is capable of being seen within Point Alpha. SCP-001-2's manifestations are random; witnesses report seeing SCP-001-2 "past the walls of the cave." From descriptions of eyewitnesses, SCP-001-2 is humanoid in shape and large in stature, and is only visible when looking through SCP-001-1. Descriptions of SCP-001-2 vary from person to person; common features include an emaciated figure, long limbs, and prominent facial features. SCP-001-2 takes the position of lying against the wall. It is most often seen with a neutral expression; however, the expression occasionally changes to discomfort or bliss.</p>\n<p>It has been determined from historical precedent that "entertaining" SCP-001-2 through SCP-001-1 will prevent SCP-001.</p>\n<div class="scp-image-block block-left" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/cave.jpg" style="width:300px;" alt="cave.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Point Alpha shortly after it was acquired by the Foundation, with Agents Samuel and Boise preforming preliminary evaluation. SCP-001-1 not visible.</p>\n</div>\n</div>\n<p><b>History:</b> Point Alpha and its contents are believed to date back to prehistory. Babylonian scripts indicate that SCP-001-1 was routinely patrolled by a group of sages, known as the "Order of White and Black," who refused passage to anyone who could demonstrate the ability to read or write.</p>\n<p>This group was made defunct by the rise of Achaemenid Persia, which contained SCP-001-1 with a group of Zoroastrian monks. Very few accounts of SCP-001-1 are present; recovered accounts assert that all information about SCP-001-1 and its significance was passed orally.</p>\n<p>The most substantial account of SCP-001-1 and SCP-001-2 at this time dates back to the Greek philosopher Xera, who made an expedition into the Achaemenid Empire and found Point Alpha. During Alexander of Macedon's conquest of the Achaemenid Empire, he took interest in Xera's texts and continued the Achaemenid containment of SCP-001-1. However, this new institution, known as the "Cronus Guard", was given Greek epics to read in order to "punish Cronus" for his actions. This institution lasted through the partition of Macedon into the Seleucid Empire, the rise of the Sassanid Empire, and even the formation of the Rashidun Caliphate.</p>\n<p>The Cronus Guard were eventually replaced with an Islamic equivalent: "The Society for the Containment of the Babel Demon." This group was the first to preform substantial research on SCP-001-1, and the first to accurately link SCP-001 to SCP-001-1.</p>\n<div class="socotbd">\n<p style="font-size: 82%"><a onclick="toggle('socotbd');">View Attachment: Excerpts from the notes of the Society for the Containment of the Babel Demon</a></p>\n</div>\n<div class="socotbd collapsed">\n <p style="font-size: 82%"><a onclick="toggle('socotbd');">\nClose Attachment</a></p>\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/book.jpg" style="width:300px;" alt="book.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Records from the Society for the Containment of the Babel Demon, recovered in the early 19<sup>th</sup> century within ORIA archives.</p>\n</div>\n</div>\n<blockquote>\n<p><b>Literature Read:</b> The first quatrain of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON squirms but otherwise does not react. Consistent with reading of Homer's Epics by the Chronus Guard.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> The remaining quatrains of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> No changes from observed behavior.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> A poem written by Guardian Muhammad ibn Buya'aa meant explicitly for DEMON</p>\n<p><b>Result:</b> DEMON thrashes more than usual until the cessation of the reading. ORB begins to expand until the <i>Genealogies of the Nobles</i> is read to DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> None, for 10 days</p>\n<p><b>Result:</b> ORB is observed to expand, and DEMON is observed to thrash. Externally, the sky above the camp is noted to turn a darker red color, until the <i>Rubaiyat of Omar Khayyam</i> is read again.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON thrashes more than during previous readings of the <i>Rubaiyat of Omar Khayyam</i>. It is believed that the Quran may no longer have any effect on DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>One Thousand and One Nights</i></p>\n<p><b>Result:</b> DEMON ceases thrashing. New literature may be necessary to prevent further phenomena from occurring.</p>\n</blockquote>\n</div>\n<p>The Society was eventually absorbed into the Office for the Reclamation of Islamic Artifacts, who assumed containment of SCP-001-1. After Incident 001-EXAL, the Foundation purchased Point Alpha from ORIA for a large sum of currency and several Safe-class anomalies.</p>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Incident 001-EXAL</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<p style="font-size: 82%"><i>Note: Some documentation taken from ORIA's account of Incident 001-EXAL.</i></p>\n<p>On <span id="today"></span>, personnel with ORIA read the book <i>Tafsir al-Ahlam al-kabir</i>, or "Great Book of Interpretation of Dreams" as per normal containment of SCP-001-1. However, SCP-001-2 was observed to rapidly thrash while screaming. SCP-001-1 itself swelled from 50.3 centimeters to 3.2 meters in diameter.</p>\n<p>Shortly following this event, SCP-001 began to worsen. The sky worldwide turned a black/white color similar to the texture of SCP-001-1. In addition, reality bending phenomena began worldwide, causing deformed geography, the manifestation of dangerous anomalous objects and several natural disasters. This event was ended after ORIA personnel read SCP-001-1 an undisclosed number of as-of-yet unread books, which caused SCP-001-2 to stop thrashing and SCP-001 to restore to its pre-incident point. However, the damage caused by this incident was deemed enough to require an activation of <a target="_top" href="http://www.scp-wiki.net/scp-2000">SCP-2000</a>.</p>\n<p>This prompted the Foundation to take control of containment of SCP-001. See <b>History</b> segment above for more information.</p>\n<div class="info">\n<p style="font-size: 82%"><a onclick="toggle('info');">Input Level 5/001 Credentials</a></p>\n</div>\n<div class="info collapsed">\n <p style="font-size: 82%"><a onclick="toggle('info');">\nAccess Granted</a></p>\n<p>During Incident 001-Alpha, vocalizations in <a target="_top" href="http://www.scp-wiki.net/scp-140">Daevish</a> were recorded by ORIA instruments within Point Alpha. The source is presently unknown. The following are approximate translations of these vocalizations.</p>\n<blockquote>\n<p>It has passed some [space/time].</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. It [has had/used to be] fun. It is [time/space] to leave.</p>\n</blockquote>\n<blockquote>\n<p>You cannot [stay/root/hold] yourself forever.</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. You [must/will] awaken.</p>\n</blockquote>\n<blockquote>\n<p>No, you have [entered/reentered] slumber for too long. Wake up.</p>\n</blockquote>\n<blockquote>\n<p>[King/Prince/loved one], it is time for you to wake up.</p>\n</blockquote>\n<blockquote>\n<p>It is fun to be in the [UNKNOWN: gaera] but you cannot be [in/rooted] there [forever/unending]. It is [entertaining/deathlike], but you must wake up.</p>\n</blockquote>\n<blockquote>\n<p>Wake up, [King/Prince/loved one]. We miss you.</p>\n</blockquote>\n</div>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Addenda</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<div class="discus">\n<p style="font-size: 82%"><a onclick="toggle('discus');">Input Level 5/001 Clearance</a></p>\n</div>\n<div class="discus collapsed">\n <p style="font-size: 82%"><a onclick="toggle('discus');">\nAccess Granted</a></p>\n<div div class="content-panel standalone series">\n<center>\n<h2>Discussion Thread 001-398:<br />Continued Containment</h2>\n<p style="font-size: 82%">Started on: <span id="today2"></span></p>\n<p style="font-size: 82%">Started by: <tt>HMCL Robinson</tt></p>\n</center>\n</div>\n<blockquote>\n<p><b><tt>HMCL Robinson:</tt></b> The usage of this thread will be for containment directives for SCP-001. In the past year that we have spent containing it, SCP-001-1's literature needs have become increasingly draconic: we can very rarely reread books without it beginning to thrash, and even then we're running out of high-quality literature to give it. We need a more long-term, reliable solution for containment. SCP-001's file has been temporarily opened up to all personnel with Level 2 clearance. Anybody can submit an idea.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Use Artificial Intelligence Constructs to automatically generate new stories for SCP-001-2.</p>\n<p><b>Status: <tt>[ <span class="green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> TSATPWTCOTTTADC.aic was able to generate 10,000 volumes of stories imitating Greek literature. However, when the first was read to SCP-001-1, it began thrashing beyond acceptable measures until it was read approved reading. No further computer-generated works are to be given to SCP-001-1.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Hire a full team of authors to create literature for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> In the past, literature written specifically for SCP-001-1 have failed to contain it (see: SCtBD Document #249). With SCP-001-2's current state this is not to be attempted again.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> This could represent a possible information leak. Test denied.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Reason:</b> With SCP-001-2's recent containment breach, these measures are approved.</p>\n<p><b>Result:</b> SCP-001-2 observed to stop thrashing and stay completely still, seemingly smiling. In addition, SCP-███ was able to be read for 14 consecutive readings before SCP-001-2 resumed normal activity.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Regularly read mainlist documentation for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Replacement of the SCP-001 file with several "001 Proposals" with much more grand implications than normal SCP files.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> By only occasionally reading 001 files, SCP-001-2 is observed to be more calm on average. The previously observed actions of retaliation in response to specially crafted literature do not apply here.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Given the variation of the current 2957 SCP mainlist files, SCP-001-1 is to be read a random arrangement of these files, in repeat.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> This strategy succeeded for approximately 3 years, allowing for several repeats. However, on <span id="yearplus3"></span>-9-30, SCP-001-2 began thrashing rapidly, requiring newer documentation to sedate.</p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-4999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> SCP-001-1 has required progressively more SCP mainlist articles for containment, even to the point of the fabrication of some anomalous entities.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Rewriting of several older SCP mainlist files to be more exaggerated and narrative-like.</p>\n<p><b>Status: <tt>[ <span style="color: orange">TENTATIVELY APPROVED</span> ]</tt></b></p>\n<p><b>Note:</b> <i>Deliberately exaggerating our files for the purpose of this entity is crossing some lines, but for now, it will have to do. However, we need a better way. I'm calling a committee. We need to put a stop to this.</i></p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Creation of a file describing SCP-001 that uses recursion to effectively be "never-ending." Psychoanalysis related to SCP-001-2's apparent "enjoyment" of works implies a slight ego and bias towards works about itself, and past strategies with this idea have worked for long periods of time. File is to be archived as a "technical" file.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> Procedure 001-ENTRY implemented. [DATA EXPUNGED]</p>\n</blockquote>\n</div>\n\n<div class="newiter">\n<p style="font-size: 82%"><a onclick="new_iteration();">Input Level 5/TECHNICAL Clearance</a></p>\n</div>\n<div class="inner">\n<!-- :-) -->\n</div>\n</div>\n</body>\n</html>\n[[/code]]\n[[iframe https://www.FILEDOMAIN/local--code/not-a-seagull-proposal/1 frameborder="0"]]
264	[[>]]\n[[module Rate]]\n[[/>]]\n[[code type="html"]]\n<html>\n<body>\n<style type="text/css">\nbody\n{\n   font-family:verdana,arial,helvetica,sans-serif;\n   font-size:12.8px;\n   color:#333;\n   line-height:141%\n}\nblockquote\n{\n  border:1px dashed #999;\n  padding: 0 12.8px;\n  background-color:#f4f4f4\n}\na {\n    color: #b01;\n    text-decoration: none;\n    background: transparent;\n}\n.content-panel {\n    border: solid 1px #888880;\n    border-radius: 10px;\n    background-color: #999990;\n    margin: 10px 0 15px;\n    box-shadow: 3px 3px 6px #bbb;\n    box-shadow: 0 2px 6px rgba(0,0,0,0.5), inset 0 1px rgba(255,255,255,0.3), inset 0 10px rgba(255,255,255,0.2), inset 0 10px 20px rgba(255,255,255,0.25), inset 0 -15px 30px rgba(0,0,0,0.1);\n}\n.content-panel.standalone {\n    background: #fcfdfb;\n}\n.content-panel.series {\n    padding: 0 20px;\n    margin-bottom: 20px;\n}\na.newpage {\n    color: #d61;\n    text-decoration: none;\n    background: transparent;\n}\na:hover {\n    text-decoration: underline;\n    background-color: transparent;\n}\n.collapsed {\n  display: none;\n}\n.vanished {\n  display: none;\n}\n.warning-cont {\n  color: red;\n  text-align: center;\n}\n#warning-header {\n  font-size: 25.6px;\n}\n#warning {\n  font-size: 20px;\n}\n.retrofont {\nfont-family: 'Geo', cursive;\nfont-weight: bold;\n}\n.cblwarning {\n  color: red;\n}\n.green {\n  color: green;\n}\n.smaller-g {\nwidth: 45%;\nmargin: auto;\nbackground-color: #e6e6e6;\nborder: none;\nheight: 2px;\n}\n.adden{\nfont-size: 1.3em;\n}\n/* Standard Image Block */\n.scp-image-block {\n    border: solid 1px #666;\n    box-shadow: 0 1px 6px rgba(0,0,0,.25);\n    width: 300px;\n}\n \n.scp-image-block.block-right {\n        float: right;\n    clear: right;\n    margin: 0 2em 1em 2em;\n}\n \n.scp-image-block.block-left {\n    float: left;\n    clear: left;\n    margin: 0 2em 1em 0;\n}\n \n.scp-image-block.block-center {\n    margin-right: auto;\n    margin-left: auto;\n}\n.scp-image-block img {\n    border: 0;\n    width: 300px;\n}\n.scp-image-block .scp-image-caption {\n    background-color: #eee;\n    border-top: solid 1px #666;\n    padding: 2px 0;\n    font-size: 80%;\n    font-weight: bold;\n    text-align: center;\n    width: 300px;\n}\n.scp-image-block > p {\n    margin: 0;\n}\n.scp-image-block .scp-image-caption > p {\n    margin: 0;\n    padding: 0 10px;\n    line-height: 125%\n}\n</style>\n<script type="text/javascript">\nfunction addEvent(element, eventName, callback) {\n    if (element.addEventListener) {\n        element.addEventListener(eventName, callback, false);\n    } else if (element.attachEvent) {\n        element.attachEvent("on" + eventName, callback);\n    } else {\n        element["on" + eventName] = callback;\n    }\n}\n\nwindow.toggle = function(cls) {\n  var collapsed = document.querySelectorAll("." + cls + ".collapsed");\n  var expanded = document.querySelectorAll("." + cls + ":not(.collapsed)");\n  var i;\n  for(i = 0; i < collapsed.length; i++) {\n    collapsed[i].classList.remove("collapsed");\n  }\n  for(i = 0; i < expanded.length; i++) {\n    expanded[i].classList.add("collapsed");\n  }\n};\n\n// typewriter code\nfunction Typewriter(el, str, delay) {\n  if (!(this instanceof Typewriter)) return new Typewriter(el, str, delay);\n\n  this.el = el;\n  this.str = str || el.innerHTML || el.value;\n  this.delay = delay || 100;\n  this.i = 0;\n}\n\nTypewriter.prototype.type = function() {\n  var i = this.i,\n    char = this.str.charAt(i);\n\n  if (!char) {\n    clearInterval(this.intervalID);\n    return this;\n  }\n\n  if (char === '<') this.isTag = true;\n  if (char === '>') this.isTag = false;\n\n  this.el.innerHTML += char;\n  return this.i++;\n};\n\nTypewriter.prototype.start = function() {\n  var self = this;\n  if (this.i < 0) this.i = 0;\n  if (this.el.innerHTML === this.str) this.clear();\n\n  // this.emit('start');\n  (function loop() {\n    self.type();\n    if (self.isTag) return loop();\n    self.intervalID = setTimeout(loop, self.delay);\n  }());\n\n  return this;\n};\n\nTypewriter.prototype.stop = function() {\n  this.i = -1;\n  return this;\n};\n\nTypewriter.prototype.restart = function() {\n  this.clear();\n  this.i = 0;\n  return this.start();\n};\n\nTypewriter.prototype.clear = function() {\n  this.el.innerHTML = '';\n  return this;\n};\n\nvar junkChars = ['ó','ò','ñ','ç','¿','ß','à','è','ì','ù','Œ','þ','█','%','+','=','.','«','»','£','$','€','½','¾','À','È','Ì','Ò','Ù','Â','Ê','â','ê','Ä','Ë','ä','ë','ÿ','ü','•'];\n\nvar an_iteration = "";\n\n\nvar randomIntFromInterval = function(min,max)\n{\n  return Math.floor(Math.random()*(max-min+1)+min);\n};\n\nvar getJunkChars = function(length) {\n  var res = '';\n  var i;\n  for (i = 0; i < length; i++) {\n    res += junkChars[randomIntFromInterval(0,junkChars.length - 1)];\n  }\n  return res;\n};\n\nvar selTabs = [];\n\nfunction rndiam() {\n  var rndiams = document.getElementsByClassName('rndiam');\n  var i;\n  for (i = 0; i < rndiams.length; i++) {\n    rndiams[i].innerHTML = (Math.random() * (1.56 - 0.51) + 0.51).toFixed(2);\n  }\n  setTimeout(rndiam, 2000);\n}\n\nwindow.onload = function() {\n\nselTabs = [\n  document.getElementsByClassName('selTab1')[0],\n  document.getElementsByClassName('selTab2')[0],\n  document.getElementsByClassName('selTab3')[0],\n  document.getElementsByClassName('selTab4')[0],\n  document.getElementsByClassName('selTab5')[0]\n];\n\nvar msg1 = '= SCP-001 | Technical Clearance Required =';\n\nvar delay1 = 50;\nvar delay2 = 25;\n\nvar toD1 = delay1 * (msg1.length);\n\nvar stopEarly = function(){\n  document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished");\n};\n\nvar typ1 =Typewriter(document.querySelectorAll('#warning-header')[0],msg1,delay1)\n\ntyp1.start();\nsetTimeout(function() { document.getElementById("warning").classList.remove("vanished"); }, toD1 + 500);\nsetTimeout(function () { document.querySelectorAll("." + "article" + ":not(.collapsed)")[0].classList.remove("vanished"); }, toD1 + 1500);\n\n  rndiam();\n\n  var today = new Date();\n  var todayformat = "" + (today.getFullYear() - 1) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today").innerHTML = todayformat;\n  var todayformat2 = "" + (today.getFullYear() - 0) + "-" + (today.getMonth() + 1) + "-" + today.getDate();\n  document.getElementById("today2").innerHTML = todayformat2;\n  document.getElementById("yearplus3").innerHTML = "" + (today.getFullYear() + 3);\n\n  an_iteration = document.getElementById("part1").innerHTML;\n};\n\nwindow.new_iteration = function() {\n  var new_iters = document.getElementsByClassName("newiter");\n  console.log(new_iters);\n  new_iters[new_iters.length - 1].classList.add("vanished");\n  var inner = document.getElementsByClassName("inner");\n  inner[inner.length - 1].innerHTML = "<hr /><hr />" + an_iteration;\n};\n</script>\n</script>\n<div class="warning-cont">\n<h2 id="warning-header">\n<p>\n</p>\n<h3 id="warning" class="vanished">  \n<p>This document exists as technical containment for an anomaly and thus does not adhere to standard SCiPNET formatting. Input 5/TECHNICAL clearance to proceed.\n</p>\n</div>\n\n<div id="collapsible_to_open_skip" class="article vanished">\n<p><center><a onclick="toggle('article');">[ INPUT PERSONAL IDENTIFICATION NUMBER ]</a></center></p>\n</div>\n<div class="article collapsed">\n <p><center><a onclick="toggle('article');">[ PERSONAL IDENTIFICATION NUMBER VERIFIED ]</a></center></p>\n<div id="part1">\n<hr />\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://topia.wikidot.com/local--files/jamcon-001/sky.jpg" style="width:300px;" alt="destruction.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>The sky above Point Alpha during a breach of SCP-001.</p>\n</div>\n</div>\n<p><b>Item #:</b> SCP-001</p>\n<p><b>Object Class:</b> Ontokinetic</p>\n<p><b>Containment Class:</b> Keter</p>\n<p><b>Special Containment Procedures:</b> If the sky begins changing color with no prior stimuli or indication, all Foundation sites are to go onto ALERT LEVEL 7 immediately. Personnel at Site-05 and -06 are to enter ALERT LEVEL 8 and begin evaluating new methods of containing SCP-001.</p>\n<p>Site-05 has been constructed around Point Alpha. Procedure 001-ENTRY is in effect to contain SCP-001-1. A monitor containing Document 032, as well as a speaker reading the document, are to be placed at least five meters away from SCP-001-1. If SCP-001-1 swells up to 2 meters or more in diameter, the speed of the narration is to increase. This system is to have a direct cable link to the Foundation SCiPNET database at Site-06, and several redundant power generators are to ensure this system stays operational at all times.</p>\n<p><b>Description:</b> SCP-001 is an ongoing ZK-Class "Reality Failure" Scenario. SCP-001 is prevented from happening through several procedures manipulating its sub-anomalies. The only known observable symptom of SCP-001 is the sky becoming a different color; it is believed that, if SCP-001 were to progress to the point where other symptoms began to occur, it would become irreversible.</p>\n<p>SCP-001-1 is a levitating object contained within Point Alpha, a cave chamber five kilometers south of the ruins of Babylon. SCP-001-1 visually resembles a perfectly smooth sphere with a texture similar to blurred television static. Measurement instruments indicate SCP-001-1 is currently <span class="rndiam"></span> meters in diameter. Solids and liquids that pass within the bounds of SCP-001-1 are replaced entirely with argon gas, giving the illusion of disappearing.</p>\n<p>SCP-001-2 is an entity that is capable of being seen within Point Alpha. SCP-001-2's manifestations are random; witnesses report seeing SCP-001-2 "past the walls of the cave." From descriptions of eyewitnesses, SCP-001-2 is humanoid in shape and large in stature, and is only visible when looking through SCP-001-1. Descriptions of SCP-001-2 vary from person to person; common features include an emaciated figure, long limbs, and prominent facial features. SCP-001-2 takes the position of lying against the wall. It is most often seen with a neutral expression; however, the expression occasionally changes to discomfort or bliss.</p>\n<p>It has been determined from historical precedent that "entertaining" SCP-001-2 through SCP-001-1 will prevent SCP-001.</p>\n<div class="scp-image-block block-left" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/cave.jpg" style="width:300px;" alt="cave.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Point Alpha shortly after it was acquired by the Foundation, with Agents Samuel and Boise preforming preliminary evaluation. SCP-001-1 not visible.</p>\n</div>\n</div>\n<p><b>History:</b> Point Alpha and its contents are believed to date back to prehistory. Babylonian scripts indicate that SCP-001-1 was routinely patrolled by a group of sages, known as the "Order of White and Black," who refused passage to anyone who could demonstrate the ability to read or write.</p>\n<p>This group was made defunct by the rise of Achaemenid Persia, which contained SCP-001-1 with a group of Zoroastrian monks. Very few accounts of SCP-001-1 are present; recovered accounts assert that all information about SCP-001-1 and its significance was passed orally.</p>\n<p>The most substantial account of SCP-001-1 and SCP-001-2 at this time dates back to the Greek philosopher Xera, who made an expedition into the Achaemenid Empire and found Point Alpha. During Alexander of Macedon's conquest of the Achaemenid Empire, he took interest in Xera's texts and continued the Achaemenid containment of SCP-001-1. However, this new institution, known as the "Cronus Guard", was given Greek epics to read in order to "punish Cronus" for his actions. This institution lasted through the partition of Macedon into the Seleucid Empire, the rise of the Sassanid Empire, and even the formation of the Rashidun Caliphate.</p>\n<p>The Cronus Guard were eventually replaced with an Islamic equivalent: "The Society for the Containment of the Babel Demon." This group was the first to preform substantial research on SCP-001-1, and the first to accurately link SCP-001 to SCP-001-1.</p>\n<div class="socotbd">\n<p style="font-size: 82%"><a onclick="toggle('socotbd');">View Attachment: Excerpts from the notes of the Society for the Containment of the Babel Demon</a></p>\n</div>\n<div class="socotbd collapsed">\n <p style="font-size: 82%"><a onclick="toggle('socotbd');">\nClose Attachment</a></p>\n<div class="scp-image-block block-right" style="width:300px;"><img src="http://scp-wiki.wdfiles.com/local--files/not-a-seagull-proposal/book.jpg" style="width:300px;" alt="book.jpg" class="image" />\n<div class="scp-image-caption" style="width:300px;">\n<p>Records from the Society for the Containment of the Babel Demon, recovered in the early 19<sup>th</sup> century within ORIA archives.</p>\n</div>\n</div>\n<blockquote>\n<p><b>Literature Read:</b> The first quatrain of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON squirms but otherwise does not react. Consistent with reading of Homer's Epics by the Chronus Guard.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> The remaining quatrains of the <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> No changes from observed behavior.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> A poem written by Guardian Muhammad ibn Buya'aa meant explicitly for DEMON</p>\n<p><b>Result:</b> DEMON thrashes more than usual until the cessation of the reading. ORB begins to expand until the <i>Genealogies of the Nobles</i> is read to DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> None, for 10 days</p>\n<p><b>Result:</b> ORB is observed to expand, and DEMON is observed to thrash. Externally, the sky above the camp is noted to turn a darker red color, until the <i>Rubaiyat of Omar Khayyam</i> is read again.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>Rubaiyat of Omar Khayyam</i></p>\n<p><b>Result:</b> DEMON thrashes more than during previous readings of the <i>Rubaiyat of Omar Khayyam</i>. It is believed that the Quran may no longer have any effect on DEMON.</p>\n</blockquote>\n<blockquote>\n<p><b>Literature Read:</b> <i>One Thousand and One Nights</i></p>\n<p><b>Result:</b> DEMON ceases thrashing. New literature may be necessary to prevent further phenomena from occurring.</p>\n</blockquote>\n</div>\n<p>The Society was eventually absorbed into the Office for the Reclamation of Islamic Artifacts, who assumed containment of SCP-001-1. After Incident 001-EXAL, the Foundation purchased Point Alpha from ORIA for a large sum of currency and several Safe-class anomalies.</p>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Incident 001-EXAL</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<p style="font-size: 82%"><i>Note: Some documentation taken from ORIA's account of Incident 001-EXAL.</i></p>\n<p>On <span id="today"></span>, personnel with ORIA read the book <i>Tafsir al-Ahlam al-kabir</i>, or "Great Book of Interpretation of Dreams" as per normal containment of SCP-001-1. However, SCP-001-2 was observed to rapidly thrash while screaming. SCP-001-1 itself swelled from 50.3 centimeters to 3.2 meters in diameter.</p>\n<p>Shortly following this event, SCP-001 began to worsen. The sky worldwide turned a black/white color similar to the texture of SCP-001-1. In addition, reality bending phenomena began worldwide, causing deformed geography, the manifestation of dangerous anomalous objects and several natural disasters. This event was ended after ORIA personnel read SCP-001-1 an undisclosed number of as-of-yet unread books, which caused SCP-001-2 to stop thrashing and SCP-001 to restore to its pre-incident point. However, the damage caused by this incident was deemed enough to require an activation of <a target="_top" href="http://www.scp-wiki.net/scp-2000">SCP-2000</a>.</p>\n<p>This prompted the Foundation to take control of containment of SCP-001. See <b>History</b> segment above for more information.</p>\n<div class="info">\n<p style="font-size: 82%"><a onclick="toggle('info');">Input Level 5/001 Credentials</a></p>\n</div>\n<div class="info collapsed">\n <p style="font-size: 82%"><a onclick="toggle('info');">\nAccess Granted</a></p>\n<p>During Incident 001-Alpha, vocalizations in <a target="_top" href="http://www.scp-wiki.net/scp-140">Daevish</a> were recorded by ORIA instruments within Point Alpha. The source is presently unknown. The following are approximate translations of these vocalizations.</p>\n<blockquote>\n<p>It has passed some [space/time].</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. It [has had/used to be] fun. It is [time/space] to leave.</p>\n</blockquote>\n<blockquote>\n<p>You cannot [stay/root/hold] yourself forever.</p>\n</blockquote>\n<blockquote>\n<p>The [UNKNOWN: gaera] has passed. You [must/will] awaken.</p>\n</blockquote>\n<blockquote>\n<p>No, you have [entered/reentered] slumber for too long. Wake up.</p>\n</blockquote>\n<blockquote>\n<p>[King/Prince/loved one], it is time for you to wake up.</p>\n</blockquote>\n<blockquote>\n<p>It is fun to be in the [UNKNOWN: gaera] but you cannot be [in/rooted] there [forever/unending]. It is [entertaining/deathlike], but you must wake up.</p>\n</blockquote>\n<blockquote>\n<p>Wake up, [King/Prince/loved one]. We miss you.</p>\n</blockquote>\n</div>\n<p>&nbsp;</p>\n<center>\n<hr class="smaller-g" />\n<p class="adden">Addenda</p>\n<hr class="smaller-g" />\n</center>\n<p>&nbsp;</p>\n<div class="discus">\n<p style="font-size: 82%"><a onclick="toggle('discus');">Input Level 5/001 Clearance</a></p>\n</div>\n<div class="discus collapsed">\n <p style="font-size: 82%"><a onclick="toggle('discus');">\nAccess Granted</a></p>\n<div div class="content-panel standalone series">\n<center>\n<h2>Discussion Thread 001-398:<br />Continued Containment</h2>\n<p style="font-size: 82%">Started on: <span id="today2"></span></p>\n<p style="font-size: 82%">Started by: <tt>HMCL Robinson</tt></p>\n</center>\n</div>\n<blockquote>\n<p><b><tt>HMCL Robinson:</tt></b> The usage of this thread will be for containment directives for SCP-001. In the past year that we have spent containing it, SCP-001-1's literature needs have become increasingly draconic: we can very rarely reread books without it beginning to thrash, and even then we're running out of high-quality literature to give it. We need a more long-term, reliable solution for containment. SCP-001's file has been temporarily opened up to all personnel with Level 2 clearance. Anybody can submit an idea.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Use Artificial Intelligence Constructs to automatically generate new stories for SCP-001-2.</p>\n<p><b>Status: <tt>[ <span class="green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> TSATPWTCOTTTADC.aic was able to generate 10,000 volumes of stories imitating Greek literature. However, when the first was read to SCP-001-1, it began thrashing beyond acceptable measures until it was read approved reading. No further computer-generated works are to be given to SCP-001-1.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Hire a full team of authors to create literature for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> In the past, literature written specifically for SCP-001-1 have failed to contain it (see: SCtBD Document #249). With SCP-001-2's current state this is not to be attempted again.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> This could represent a possible information leak. Test denied.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Read mainlist documentation for SCP-███ to SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Reason:</b> With SCP-001-2's recent containment breach, these measures are approved.</p>\n<p><b>Result:</b> SCP-001-2 observed to stop thrashing and stay completely still, seemingly smiling. In addition, SCP-███ was able to be read for 14 consecutive readings before SCP-001-2 resumed normal activity.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Regularly read mainlist documentation for SCP-001-1.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Replacement of the SCP-001 file with several "001 Proposals" with much more grand implications than normal SCP files.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> By only occasionally reading 001 files, SCP-001-2 is observed to be more calm on average. The previously observed actions of retaliation in response to specially crafted literature do not apply here.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Given the variation of the current 2957 SCP mainlist files, SCP-001-1 is to be read a random arrangement of these files, in repeat.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> This strategy succeeded for approximately 3 years, allowing for several repeats. However, on <span id="yearplus3"></span>-9-30, SCP-001-2 began thrashing rapidly, requiring newer documentation to sedate.</p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Expansion of the SCP Series beyond SCP-4999, and declassification of several unnumbered SCP objects into these slots.</p>\n<p><b>Status: <tt>[ <span style="color: red">DENIED</span> ]</tt></b></p>\n<p><b>Reason:</b> SCP-001-1 has required progressively more SCP mainlist articles for containment, even to the point of the fabrication of some anomalous entities.</p>\n</blockquote>\n<blockquote>\n<p><b>Proposal:</b> Rewriting of several older SCP mainlist files to be more exaggerated and narrative-like.</p>\n<p><b>Status: <tt>[ <span style="color: orange">TENTATIVELY APPROVED</span> ]</tt></b></p>\n<p><b>Note:</b> <i>Deliberately exaggerating our files for the purpose of this entity is crossing some lines, but for now, it will have to do. However, we need a better way. I'm calling a committee. We need to put a stop to this.</i></p>\n</blockquote>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<p>&nbsp;</p>\n<blockquote>\n<p><b>Proposal:</b> Creation of a file describing SCP-001 that uses recursion to effectively be "never-ending." Psychoanalysis related to SCP-001-2's apparent "enjoyment" of works implies a slight ego and bias towards works about itself, and past strategies with this idea have worked for long periods of time. File is to be archived as a "technical" file.</p>\n<p><b>Status: <tt>[ <span style="color: green">APPROVED</span> ]</tt></b></p>\n<p><b>Result:</b> Procedure 001-ENTRY implemented. [DATA EXPUNGED]</p>\n</blockquote>\n</div>\n\n<div class="newiter">\n<p style="font-size: 82%"><a onclick="new_iteration();">Input Level 5/TECHNICAL Clearance</a></p>\n</div>\n<div class="inner">\n<!-- :-) -->\n</div>\n</div>\n</body>\n</html>\n[[/code]]\n[[iframe https://www.FILEDOMAIN/local--code/not-a-seagull-proposal/1 frameborder="0" width="100%"]]
265	* [[[start | Welcome page]]]\n\n* [[[What is a Wiki Site?]]]\n* [[[How to edit pages?]]]\n\n* [[[system: join | How to join this site?]]]\n* [[[system:members | Site members]]] \n\n* [[[system: Recent changes]]]\n* [[[system: List all pages]]]\n* [[[system:page-tags-list|Page Tags]]]\n\n* [[[admin:manage|Site Manager]]]\n\n++ Page tags\n[[module TagCloud minFontSize="80%" maxFontSize="200%"  maxColor="8,8,64" minColor="100,100,128" target="system:page-tags" limit="30"]]\n\n++ Add a new page\n[[module NewPage size="15" button="new page"]]\n\n= [[size 80%]][[[nav:side | edit this panel]]][[/size]]
266	* [# Sample Menu]\n * [[[www::start|MyWikidot Home]]]\n * [[[www::mywikidot-info|Experienced users]]]\n* [# Edit/Print]\n * [[button edit text="Edit This Page"]]\n * [[button print text="Print This Page"]]\n* [# Admin]\n * [[[nav:top|Edit Top Navigation]]]\n * [[[nav:side|Edit Side Navigation]]]\n * [[[admin:manage|Site Manager]]]\n\n[!-- top nav menu, use only one bulleted list above --]
267	[[module Search]]\n\n[!-- please do not remove or change this page if you want to keep the search function working --]
268	According to [http://en.wikipedia.org/wiki/Wiki Wikipedia], the world largest wiki site:\n\n> A //Wiki// ([ˈwiː.kiː] <wee-kee> or [ˈwɪ.kiː] <wick-ey>) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.\n\nAnd that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.
269	++ If this is your first site\n\nThen there are some things you need to know:\n\n* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.\n* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called '{{nav:side}}', and [[[nav:top | one at the top]]] called '{{nav:top}}'.  These are Wikidot pages, and you can edit them like any page.\n* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].\n* You can attach images and other files to any page, then display them and link to them in the page.\n* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.\n* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.\n* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.\n* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]\n\nMore information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org].
270	[!--\nHow To Edit Pages - Quickstart\n--]\nIf you are allowed to edit pages in this Site, simply click on //edit// button at the bottom of the page. This will open an editor with a toolbar pallette with options.\n\nTo create a link to a new page, use syntax: {{``[[[new page name]]]``}} or {{``[[[new page name | text to display]]]``}}. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!\n\nAlthough creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit [*http://www.wikidot.org/doc Documentation pages] (at wikidot.org) to learn more.
271	[[module ManageSite]]
272	[[note]]\nPlease change this page according to your policy (configure first using [[[admin:manage|Site Manager]]]) and remove this note.\n[[/note]]\n\n+ Who can join?\n\nYou can write here who can become a member of this site.\n\n+ Join!\n\nSo you want to become a member of this site? Tell us why and apply now!\n\n[[module MembershipApply]] \n\nOr, if you already know a "secret password", go for it!\n\n[[module MembershipByPassword]]
273	[[module TagCloud limit="200" target="system:page-tags"]]\n\n[!--\n\nYou can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module \nBut if you want to keep the tag functionality working - do not remove these modules.\n\n--]
274	[[module SiteChanges]]
282	hi im storm
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
5	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	0
6	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	0
7		f	\N	\N	\N			http://scp-wiki.net/aismallard								0
8	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	0
9	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	0
10	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	\N	0
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
28	Black Highlighter	bhl	f	1	\N	t	5	t	t	0		0
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
5	t	a    	*	*	\N	t	t	t	3
6	t	a    	*	*	\N	t	t	t	3
7	t	a    	*	*	\N	t	t	t	3
8	t	a    	*	*	\N	t	t	t	3
9	t	a    	*	*	\N	t	t	t	3
10	t	a    	*	*	\N	t	t	t	3
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
