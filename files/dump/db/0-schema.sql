


CREATE TABLE admin (
    admin_id integer NOT NULL,
    site_id integer,
    user_id integer,
    founder boolean DEFAULT false
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE admin_admin_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE admin_admin_id_seq OWNED BY admin.admin_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('admin_admin_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE admin_notification (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE admin_notification_notification_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE admin_notification_notification_id_seq OWNED BY admin_notification.notification_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('admin_notification_notification_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE anonymous_abuse_flag (
    flag_id integer NOT NULL,
    user_id integer,
    address inet,
    proxy boolean DEFAULT false,
    site_id integer,
    site_valid boolean DEFAULT true,
    global_valid boolean DEFAULT true
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE anonymous_abuse_flag_flag_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE anonymous_abuse_flag_flag_id_seq OWNED BY anonymous_abuse_flag.flag_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('anonymous_abuse_flag_flag_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE api_key (
    key character varying(64) NOT NULL,
    user_id integer
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE category (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE category_category_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE category_category_id_seq OWNED BY category.category_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('category_category_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE category_template (
    category_template_id integer NOT NULL,
    source text
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE category_template_category_template_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE category_template_category_template_id_seq OWNED BY category_template.category_template_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('category_template_category_template_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE comment (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE comment_comment_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE comment_comment_id_seq OWNED BY comment.comment_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('comment_comment_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE comment_revision (
    revision_id integer NOT NULL,
    comment_id integer,
    user_id integer,
    user_string character varying(80),
    text text,
    title character varying(256),
    date timestamp without time zone
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE comment_revision_revision_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE comment_revision_revision_id_seq OWNED BY comment_revision.revision_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('comment_revision_revision_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE contact (
    contact_id integer NOT NULL,
    user_id integer,
    target_user_id integer
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE contact_contact_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE contact_contact_id_seq OWNED BY contact.contact_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('contact_contact_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE domain_redirect (
    redirect_id integer NOT NULL,
    site_id integer,
    url character varying(80)
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE domain_redirect_redirect_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE domain_redirect_redirect_id_seq OWNED BY domain_redirect.redirect_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('domain_redirect_redirect_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE email_invitation (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE email_invitation_invitation_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE email_invitation_invitation_id_seq OWNED BY email_invitation.invitation_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('email_invitation_invitation_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE file (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE file_file_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE file_file_id_seq OWNED BY file.file_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('file_file_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE files_event (
    file_event_id integer NOT NULL,
    filename character varying(100),
    date timestamp without time zone,
    user_id integer,
    user_string character varying(80),
    action character varying(80),
    action_extra character varying(80)
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE files_event_file_event_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE files_event_file_event_id_seq OWNED BY files_event.file_event_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('files_event_file_event_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE form_submission_key (
    key_id character varying(90) NOT NULL,
    date_submitted timestamp without time zone
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE forum_category (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE forum_category_category_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE forum_category_category_id_seq OWNED BY forum_category.category_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('forum_category_category_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE forum_group (
    group_id integer NOT NULL,
    name character varying(80),
    description text,
    sort_index integer DEFAULT 0,
    site_id integer,
    visible boolean DEFAULT true
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE forum_group_group_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE forum_group_group_id_seq OWNED BY forum_group.group_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('forum_group_group_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE forum_post (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE forum_post_post_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE forum_post_post_id_seq OWNED BY forum_post.post_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('forum_post_post_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE forum_post_revision (
    revision_id integer NOT NULL,
    post_id integer,
    user_id integer,
    user_string character varying(80),
    text text,
    title character varying(256),
    date timestamp without time zone
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE forum_post_revision_revision_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE forum_post_revision_revision_id_seq OWNED BY forum_post_revision.revision_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('forum_post_revision_revision_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE forum_settings (
    site_id integer NOT NULL,
    permissions character varying(200),
    per_page_discussion boolean DEFAULT false,
    max_nest_level integer DEFAULT 0
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE forum_thread (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE forum_thread_thread_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE forum_thread_thread_id_seq OWNED BY forum_thread.thread_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('forum_thread_thread_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE front_forum_feed (
    feed_id integer NOT NULL,
    page_id integer,
    title character varying(90),
    label character varying(90),
    description character varying(256),
    categories character varying(100),
    parmhash character varying(100),
    site_id integer
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE front_forum_feed_feed_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE front_forum_feed_feed_id_seq OWNED BY front_forum_feed.feed_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('front_forum_feed_feed_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE fts_entry (
    fts_id integer NOT NULL,
    page_id integer,
    title character varying(256),
    unix_name character varying(100),
    thread_id integer,
    site_id integer,
    text text,
    vector tsvector
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE fts_entry_fts_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE fts_entry_fts_id_seq OWNED BY fts_entry.fts_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('fts_entry_fts_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE global_ip_block (
    block_id integer NOT NULL,
    address inet,
    flag_proxy boolean DEFAULT false,
    reason text,
    flag_total boolean DEFAULT false,
    date_blocked timestamp without time zone
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE global_ip_block_block_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE global_ip_block_block_id_seq OWNED BY global_ip_block.block_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('global_ip_block_block_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE global_user_block (
    block_id integer NOT NULL,
    site_id integer,
    user_id integer,
    reason text,
    date_blocked timestamp without time zone
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE global_user_block_block_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE global_user_block_block_id_seq OWNED BY global_user_block.block_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('global_user_block_block_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE ip_block (
    block_id integer NOT NULL,
    site_id integer,
    ip inet,
    flag_proxy boolean DEFAULT false,
    reason text,
    date_blocked timestamp without time zone
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE ip_block_block_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE ip_block_block_id_seq OWNED BY ip_block.block_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('ip_block_block_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE license (
    license_id integer NOT NULL,
    name character varying(100),
    description text,
    sort integer DEFAULT 0
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE license_license_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE license_license_id_seq OWNED BY license.license_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('license_license_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE log_event (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE log_event_event_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE log_event_event_id_seq OWNED BY log_event.event_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('log_event_event_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE member (
    member_id integer NOT NULL,
    site_id integer,
    user_id integer,
    date_joined timestamp without time zone,
    allow_newsletter boolean DEFAULT true
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE member_application (
    application_id integer NOT NULL,
    site_id integer,
    user_id integer,
    status character varying(20) DEFAULT 'pending'::character varying,
    date timestamp without time zone,
    comment text,
    reply text
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE member_application_application_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE member_application_application_id_seq OWNED BY member_application.application_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('member_application_application_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE member_invitation (
    invitation_id integer NOT NULL,
    site_id integer,
    user_id integer,
    by_user_id integer,
    date timestamp without time zone,
    body text
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE member_invitation_invitation_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE member_invitation_invitation_id_seq OWNED BY member_invitation.invitation_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('member_invitation_invitation_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE SEQUENCE member_member_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE member_member_id_seq OWNED BY member.member_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('member_member_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE membership_link (
    link_id integer NOT NULL,
    site_id integer,
    by_user_id integer,
    user_id integer,
    date timestamp without time zone,
    type character varying(20)
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE membership_link_link_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE membership_link_link_id_seq OWNED BY membership_link.link_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('membership_link_link_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE moderator (
    moderator_id integer NOT NULL,
    site_id integer,
    user_id integer,
    permissions character(10)
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE moderator_moderator_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE moderator_moderator_id_seq OWNED BY moderator.moderator_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('moderator_moderator_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE notification (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE notification_notification_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE notification_notification_id_seq OWNED BY notification.notification_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('notification_notification_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE openid_entry (
    openid_id integer NOT NULL,
    site_id integer,
    page_id integer,
    type character varying(10),
    user_id integer,
    url character varying(100),
    server_url character varying(100)
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE openid_entry_openid_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE openid_entry_openid_id_seq OWNED BY openid_entry.openid_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('openid_entry_openid_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE ozone_group (
    group_id integer NOT NULL,
    parent_group_id integer,
    name character varying(50),
    description text
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE ozone_group_group_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE ozone_group_group_id_seq OWNED BY ozone_group.group_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('ozone_group_group_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE ozone_group_permission_modifier (
    group_permission_id integer NOT NULL,
    group_id character varying(20),
    permission_id character varying(20),
    modifier integer
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE ozone_group_permission_modifier_group_permission_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE ozone_group_permission_modifier_group_permission_id_seq OWNED BY ozone_group_permission_modifier.group_permission_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('ozone_group_permission_modifier_group_permission_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE ozone_lock (
    key character varying(100) NOT NULL
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE ozone_permission (
    permission_id integer NOT NULL,
    name character varying(50),
    description text
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE ozone_permission_permission_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE ozone_permission_permission_id_seq OWNED BY ozone_permission.permission_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('ozone_permission_permission_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE ozone_session (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE ozone_user (
    user_id integer NOT NULL,
    name character varying(99),
    nick_name character varying(70),
    password character varying(99),
    email character varying(99),
    unix_name character varying(99),
    last_login timestamp without time zone,
    registered_date timestamp without time zone,
    super_admin boolean DEFAULT false,
    super_moderator boolean DEFAULT false,
    language character varying(10) DEFAULT 'en'::character varying
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE ozone_user_group_relation (
    user_group_id integer NOT NULL,
    user_id integer,
    group_id integer
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE ozone_user_group_relation_user_group_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE ozone_user_group_relation_user_group_id_seq OWNED BY ozone_user_group_relation.user_group_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('ozone_user_group_relation_user_group_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE ozone_user_permission_modifier (
    user_permission_id integer NOT NULL,
    user_id integer,
    permission_id character varying(20),
    modifier integer
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE ozone_user_permission_modifier_user_permission_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE ozone_user_permission_modifier_user_permission_id_seq OWNED BY ozone_user_permission_modifier.user_permission_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('ozone_user_permission_modifier_user_permission_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE SEQUENCE ozone_user_user_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE ozone_user_user_id_seq OWNED BY ozone_user.user_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('ozone_user_user_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE page (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE page_abuse_flag (
    flag_id integer NOT NULL,
    user_id integer,
    site_id integer,
    path character varying(100),
    site_valid boolean DEFAULT true,
    global_valid boolean DEFAULT true
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE page_abuse_flag_flag_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE page_abuse_flag_flag_id_seq OWNED BY page_abuse_flag.flag_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('page_abuse_flag_flag_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE page_compiled (
    page_id integer NOT NULL,
    text text,
    date_compiled timestamp without time zone
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE page_edit_lock (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE page_edit_lock_lock_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE page_edit_lock_lock_id_seq OWNED BY page_edit_lock.lock_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('page_edit_lock_lock_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE page_external_link (
    link_id integer NOT NULL,
    site_id integer,
    page_id integer,
    to_url character varying(512),
    pinged boolean DEFAULT false,
    ping_status character varying(256),
    date timestamp without time zone
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE page_external_link_link_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE page_external_link_link_id_seq OWNED BY page_external_link.link_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('page_external_link_link_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE page_inclusion (
    inclusion_id integer NOT NULL,
    including_page_id integer,
    included_page_id integer,
    included_page_name character varying(128),
    site_id integer
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE page_inclusion_inclusion_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE page_inclusion_inclusion_id_seq OWNED BY page_inclusion.inclusion_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('page_inclusion_inclusion_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE page_link (
    link_id integer NOT NULL,
    from_page_id integer,
    to_page_id integer,
    to_page_name character varying(128),
    site_id integer
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE page_link_link_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE page_link_link_id_seq OWNED BY page_link.link_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('page_link_link_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE page_metadata (
    metadata_id integer NOT NULL,
    parent_page_id integer,
    title character varying(256),
    unix_name character varying(80),
    owner_user_id integer
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE page_metadata_metadata_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE page_metadata_metadata_id_seq OWNED BY page_metadata.metadata_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('page_metadata_metadata_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE SEQUENCE page_page_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE page_page_id_seq OWNED BY page.page_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('page_page_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE page_rate_vote (
    rate_id integer NOT NULL,
    user_id integer,
    page_id integer,
    rate integer DEFAULT 1,
    date timestamp without time zone
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE page_rate_vote_rate_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE page_rate_vote_rate_id_seq OWNED BY page_rate_vote.rate_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('page_rate_vote_rate_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE page_revision (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE page_revision_revision_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE page_revision_revision_id_seq OWNED BY page_revision.revision_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('page_revision_revision_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE page_source (
    source_id integer NOT NULL,
    text text
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE page_source_source_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE page_source_source_id_seq OWNED BY page_source.source_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('page_source_source_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE page_tag (
    tag_id bigint NOT NULL,
    site_id integer,
    page_id integer,
    tag character varying(20)
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE page_tag_tag_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE page_tag_tag_id_seq OWNED BY page_tag.tag_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('page_tag_tag_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE petition_campaign (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE petition_campaign_campaign_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE petition_campaign_campaign_id_seq OWNED BY petition_campaign.campaign_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('petition_campaign_campaign_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE petition_signature (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE petition_signature_signature_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE petition_signature_signature_id_seq OWNED BY petition_signature.signature_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('petition_signature_signature_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE private_message (
    message_id integer NOT NULL,
    from_user_id integer,
    to_user_id integer,
    subject character varying(256),
    body text,
    date timestamp without time zone,
    flag integer DEFAULT 0,
    flag_new boolean DEFAULT true
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE private_message_message_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE private_message_message_id_seq OWNED BY private_message.message_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('private_message_message_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE private_user_block (
    block_id integer NOT NULL,
    user_id integer,
    blocked_user_id integer
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE private_user_block_block_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE private_user_block_block_id_seq OWNED BY private_user_block.block_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('private_user_block_block_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE profile (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE simpletodo_list (
    list_id integer NOT NULL,
    site_id integer,
    label character varying(256),
    title character varying(256),
    data text
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE simpletodo_list_list_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE simpletodo_list_list_id_seq OWNED BY simpletodo_list.list_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('simpletodo_list_list_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE site (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE site_backup (
    backup_id integer NOT NULL,
    site_id integer,
    status character varying(50),
    backup_source boolean DEFAULT true,
    backup_files boolean DEFAULT true,
    date timestamp without time zone,
    rand character varying(100)
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE site_backup_backup_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE site_backup_backup_id_seq OWNED BY site_backup.backup_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('site_backup_backup_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE site_settings (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE site_site_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE site_site_id_seq OWNED BY site.site_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('site_site_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE site_super_settings (
    site_id integer NOT NULL,
    can_custom_domain boolean DEFAULT true
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE site_tag (
    tag_id integer NOT NULL,
    site_id integer,
    tag character varying(20)
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE site_tag_tag_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE site_tag_tag_id_seq OWNED BY site_tag.tag_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('site_tag_tag_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE site_viewer (
    viewer_id integer NOT NULL,
    site_id integer,
    user_id integer
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE site_viewer_viewer_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE site_viewer_viewer_id_seq OWNED BY site_viewer.viewer_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('site_viewer_viewer_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE storage_item (
    item_id character varying(256) NOT NULL,
    date timestamp without time zone,
    timeout integer,
    data bytea
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE theme (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE theme_preview (
    theme_id integer NOT NULL,
    body text
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE theme_theme_id_seq
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE theme_theme_id_seq OWNED BY theme.theme_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('theme_theme_id_seq', 100, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE ucookie (
    ucookie_id character varying(100) NOT NULL,
    site_id integer,
    session_id character varying(60),
    date_granted timestamp without time zone
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE unique_string_broker (
    last_index integer
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE user_abuse_flag (
    flag_id integer NOT NULL,
    user_id integer,
    target_user_id integer,
    site_id integer,
    site_valid boolean DEFAULT true,
    global_valid boolean DEFAULT true
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE user_abuse_flag_flag_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE user_abuse_flag_flag_id_seq OWNED BY user_abuse_flag.flag_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('user_abuse_flag_flag_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE user_block (
    block_id integer NOT NULL,
    site_id integer,
    user_id integer,
    reason text,
    date_blocked timestamp without time zone
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE user_block_block_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE user_block_block_id_seq OWNED BY user_block.block_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('user_block_block_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE user_karma (
    user_id integer NOT NULL,
    points integer DEFAULT 0,
    level integer DEFAULT 0
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE user_settings (
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
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE TABLE watched_forum_thread (
    watched_id integer NOT NULL,
    user_id integer,
    thread_id integer
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE watched_forum_thread_watched_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE watched_forum_thread_watched_id_seq OWNED BY watched_forum_thread.watched_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('watched_forum_thread_watched_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE TABLE watched_page (
    watched_id integer NOT NULL,
    user_id integer,
    page_id integer
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




CREATE SEQUENCE watched_page_watched_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MAXVALUE
    NO MINVALUE
    CACHE 1;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;




ALTER SEQUENCE watched_page_watched_id_seq OWNED BY watched_page.watched_id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



SELECT pg_catalog.setval('watched_page_watched_id_seq', 100, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE admin ALTER COLUMN admin_id SET DEFAULT nextval('admin_admin_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE admin_notification ALTER COLUMN notification_id SET DEFAULT nextval('admin_notification_notification_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE anonymous_abuse_flag ALTER COLUMN flag_id SET DEFAULT nextval('anonymous_abuse_flag_flag_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE category ALTER COLUMN category_id SET DEFAULT nextval('category_category_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE category_template ALTER COLUMN category_template_id SET DEFAULT nextval('category_template_category_template_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE comment ALTER COLUMN comment_id SET DEFAULT nextval('comment_comment_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE comment_revision ALTER COLUMN revision_id SET DEFAULT nextval('comment_revision_revision_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE contact ALTER COLUMN contact_id SET DEFAULT nextval('contact_contact_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE domain_redirect ALTER COLUMN redirect_id SET DEFAULT nextval('domain_redirect_redirect_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE email_invitation ALTER COLUMN invitation_id SET DEFAULT nextval('email_invitation_invitation_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE file ALTER COLUMN file_id SET DEFAULT nextval('file_file_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE files_event ALTER COLUMN file_event_id SET DEFAULT nextval('files_event_file_event_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE forum_category ALTER COLUMN category_id SET DEFAULT nextval('forum_category_category_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE forum_group ALTER COLUMN group_id SET DEFAULT nextval('forum_group_group_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE forum_post ALTER COLUMN post_id SET DEFAULT nextval('forum_post_post_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE forum_post_revision ALTER COLUMN revision_id SET DEFAULT nextval('forum_post_revision_revision_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE forum_thread ALTER COLUMN thread_id SET DEFAULT nextval('forum_thread_thread_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE front_forum_feed ALTER COLUMN feed_id SET DEFAULT nextval('front_forum_feed_feed_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE fts_entry ALTER COLUMN fts_id SET DEFAULT nextval('fts_entry_fts_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE global_ip_block ALTER COLUMN block_id SET DEFAULT nextval('global_ip_block_block_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE global_user_block ALTER COLUMN block_id SET DEFAULT nextval('global_user_block_block_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ip_block ALTER COLUMN block_id SET DEFAULT nextval('ip_block_block_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE license ALTER COLUMN license_id SET DEFAULT nextval('license_license_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE log_event ALTER COLUMN event_id SET DEFAULT nextval('log_event_event_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE member ALTER COLUMN member_id SET DEFAULT nextval('member_member_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE member_application ALTER COLUMN application_id SET DEFAULT nextval('member_application_application_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE member_invitation ALTER COLUMN invitation_id SET DEFAULT nextval('member_invitation_invitation_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE membership_link ALTER COLUMN link_id SET DEFAULT nextval('membership_link_link_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE moderator ALTER COLUMN moderator_id SET DEFAULT nextval('moderator_moderator_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE notification ALTER COLUMN notification_id SET DEFAULT nextval('notification_notification_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE openid_entry ALTER COLUMN openid_id SET DEFAULT nextval('openid_entry_openid_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ozone_group ALTER COLUMN group_id SET DEFAULT nextval('ozone_group_group_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ozone_group_permission_modifier ALTER COLUMN group_permission_id SET DEFAULT nextval('ozone_group_permission_modifier_group_permission_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ozone_permission ALTER COLUMN permission_id SET DEFAULT nextval('ozone_permission_permission_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ozone_user ALTER COLUMN user_id SET DEFAULT nextval('ozone_user_user_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ozone_user_group_relation ALTER COLUMN user_group_id SET DEFAULT nextval('ozone_user_group_relation_user_group_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ozone_user_permission_modifier ALTER COLUMN user_permission_id SET DEFAULT nextval('ozone_user_permission_modifier_user_permission_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE page ALTER COLUMN page_id SET DEFAULT nextval('page_page_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE page_abuse_flag ALTER COLUMN flag_id SET DEFAULT nextval('page_abuse_flag_flag_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE page_edit_lock ALTER COLUMN lock_id SET DEFAULT nextval('page_edit_lock_lock_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE page_external_link ALTER COLUMN link_id SET DEFAULT nextval('page_external_link_link_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE page_inclusion ALTER COLUMN inclusion_id SET DEFAULT nextval('page_inclusion_inclusion_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE page_link ALTER COLUMN link_id SET DEFAULT nextval('page_link_link_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE page_metadata ALTER COLUMN metadata_id SET DEFAULT nextval('page_metadata_metadata_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE page_rate_vote ALTER COLUMN rate_id SET DEFAULT nextval('page_rate_vote_rate_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE page_revision ALTER COLUMN revision_id SET DEFAULT nextval('page_revision_revision_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE page_source ALTER COLUMN source_id SET DEFAULT nextval('page_source_source_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE page_tag ALTER COLUMN tag_id SET DEFAULT nextval('page_tag_tag_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE petition_campaign ALTER COLUMN campaign_id SET DEFAULT nextval('petition_campaign_campaign_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE petition_signature ALTER COLUMN signature_id SET DEFAULT nextval('petition_signature_signature_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE private_message ALTER COLUMN message_id SET DEFAULT nextval('private_message_message_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE private_user_block ALTER COLUMN block_id SET DEFAULT nextval('private_user_block_block_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE simpletodo_list ALTER COLUMN list_id SET DEFAULT nextval('simpletodo_list_list_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE site ALTER COLUMN site_id SET DEFAULT nextval('site_site_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE site_backup ALTER COLUMN backup_id SET DEFAULT nextval('site_backup_backup_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE site_tag ALTER COLUMN tag_id SET DEFAULT nextval('site_tag_tag_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE site_viewer ALTER COLUMN viewer_id SET DEFAULT nextval('site_viewer_viewer_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE theme ALTER COLUMN theme_id SET DEFAULT nextval('theme_theme_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE user_abuse_flag ALTER COLUMN flag_id SET DEFAULT nextval('user_abuse_flag_flag_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE user_block ALTER COLUMN block_id SET DEFAULT nextval('user_block_block_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE watched_forum_thread ALTER COLUMN watched_id SET DEFAULT nextval('watched_forum_thread_watched_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE watched_page ALTER COLUMN watched_id SET DEFAULT nextval('watched_page_watched_id_seq'::regclass);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY admin
    ADD CONSTRAINT admin__site_id__user_id__unique UNIQUE (site_id, user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY admin_notification
    ADD CONSTRAINT admin_notification_pkey PRIMARY KEY (notification_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY admin
    ADD CONSTRAINT admin_pkey PRIMARY KEY (admin_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY anonymous_abuse_flag
    ADD CONSTRAINT anonymous_abuse_flag_pkey PRIMARY KEY (flag_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY api_key
    ADD CONSTRAINT api_key_pkey PRIMARY KEY (key);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY category
    ADD CONSTRAINT category_pkey PRIMARY KEY (category_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY category_template
    ADD CONSTRAINT category_template_pkey PRIMARY KEY (category_template_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY comment
    ADD CONSTRAINT comment_pkey PRIMARY KEY (comment_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY comment_revision
    ADD CONSTRAINT comment_revision_pkey PRIMARY KEY (revision_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY contact
    ADD CONSTRAINT contact__unique UNIQUE (user_id, target_user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY contact
    ADD CONSTRAINT contact_pkey PRIMARY KEY (contact_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY domain_redirect
    ADD CONSTRAINT domain_redirect__unique UNIQUE (site_id, url);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY domain_redirect
    ADD CONSTRAINT domain_redirect_pkey PRIMARY KEY (redirect_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY email_invitation
    ADD CONSTRAINT email_invitation_pkey PRIMARY KEY (invitation_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY file
    ADD CONSTRAINT file_pkey PRIMARY KEY (file_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY files_event
    ADD CONSTRAINT files_event_pkey PRIMARY KEY (file_event_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY form_submission_key
    ADD CONSTRAINT form_submission_key_pkey PRIMARY KEY (key_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_category
    ADD CONSTRAINT forum_category_pkey PRIMARY KEY (category_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_group
    ADD CONSTRAINT forum_group_pkey PRIMARY KEY (group_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_post
    ADD CONSTRAINT forum_post_pkey PRIMARY KEY (post_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_post_revision
    ADD CONSTRAINT forum_post_revision_pkey PRIMARY KEY (revision_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_settings
    ADD CONSTRAINT forum_settings_pkey PRIMARY KEY (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_thread
    ADD CONSTRAINT forum_thread_pkey PRIMARY KEY (thread_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY front_forum_feed
    ADD CONSTRAINT front_forum_feed_pkey PRIMARY KEY (feed_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY fts_entry
    ADD CONSTRAINT fts_entry_pkey PRIMARY KEY (fts_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY global_ip_block
    ADD CONSTRAINT global_ip_block_pkey PRIMARY KEY (block_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY global_user_block
    ADD CONSTRAINT global_user_block_pkey PRIMARY KEY (block_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ip_block
    ADD CONSTRAINT ip_block_pkey PRIMARY KEY (block_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY license
    ADD CONSTRAINT license_name_key UNIQUE (name);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY license
    ADD CONSTRAINT license_pkey PRIMARY KEY (license_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY log_event
    ADD CONSTRAINT log_event_pkey PRIMARY KEY (event_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY member
    ADD CONSTRAINT member__unique UNIQUE (site_id, user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY member_application
    ADD CONSTRAINT member_application__unique UNIQUE (site_id, user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY member_application
    ADD CONSTRAINT member_application_pkey PRIMARY KEY (application_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY member_invitation
    ADD CONSTRAINT member_invitation_pkey PRIMARY KEY (invitation_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY member
    ADD CONSTRAINT member_pkey PRIMARY KEY (member_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY membership_link
    ADD CONSTRAINT membership_link_pkey PRIMARY KEY (link_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY moderator
    ADD CONSTRAINT moderator__unique UNIQUE (site_id, user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY moderator
    ADD CONSTRAINT moderator_pkey PRIMARY KEY (moderator_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY notification
    ADD CONSTRAINT notification_pkey PRIMARY KEY (notification_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY openid_entry
    ADD CONSTRAINT openid_entry_pkey PRIMARY KEY (openid_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ozone_group
    ADD CONSTRAINT ozone_group_name_key UNIQUE (name);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ozone_group_permission_modifier
    ADD CONSTRAINT ozone_group_permission_modifier_pkey PRIMARY KEY (group_permission_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ozone_group
    ADD CONSTRAINT ozone_group_pkey PRIMARY KEY (group_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ozone_lock
    ADD CONSTRAINT ozone_lock_pkey PRIMARY KEY (key);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ozone_permission
    ADD CONSTRAINT ozone_permission_name_key UNIQUE (name);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ozone_permission
    ADD CONSTRAINT ozone_permission_pkey PRIMARY KEY (permission_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ozone_session
    ADD CONSTRAINT ozone_session_pkey PRIMARY KEY (session_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ozone_user_group_relation
    ADD CONSTRAINT ozone_user_group_relation_pkey PRIMARY KEY (user_group_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ozone_user
    ADD CONSTRAINT ozone_user_name_key UNIQUE (name);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ozone_user_permission_modifier
    ADD CONSTRAINT ozone_user_permission_modifier_pkey PRIMARY KEY (user_permission_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ozone_user
    ADD CONSTRAINT ozone_user_pkey PRIMARY KEY (user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ozone_user
    ADD CONSTRAINT ozone_user_unix_name_key UNIQUE (unix_name);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page
    ADD CONSTRAINT page__unique UNIQUE (site_id, unix_name);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_abuse_flag
    ADD CONSTRAINT page_abuse_flag_pkey PRIMARY KEY (flag_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_compiled
    ADD CONSTRAINT page_compiled_pkey PRIMARY KEY (page_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_edit_lock
    ADD CONSTRAINT page_edit_lock_pkey PRIMARY KEY (lock_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_external_link
    ADD CONSTRAINT page_external_link_pkey PRIMARY KEY (link_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_inclusion
    ADD CONSTRAINT page_inclusion__unique UNIQUE (including_page_id, included_page_id, included_page_name);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_inclusion
    ADD CONSTRAINT page_inclusion_pkey PRIMARY KEY (inclusion_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_link
    ADD CONSTRAINT page_link__unique UNIQUE (from_page_id, to_page_id, to_page_name);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_link
    ADD CONSTRAINT page_link_pkey PRIMARY KEY (link_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_metadata
    ADD CONSTRAINT page_metadata_pkey PRIMARY KEY (metadata_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page
    ADD CONSTRAINT page_pkey PRIMARY KEY (page_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_rate_vote
    ADD CONSTRAINT page_rate_vote_pkey PRIMARY KEY (rate_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_rate_vote
    ADD CONSTRAINT page_rate_vote_user_id_key UNIQUE (user_id, page_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_revision
    ADD CONSTRAINT page_revision_pkey PRIMARY KEY (revision_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_source
    ADD CONSTRAINT page_source_pkey PRIMARY KEY (source_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_tag
    ADD CONSTRAINT page_tag_pkey PRIMARY KEY (tag_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY petition_campaign
    ADD CONSTRAINT petition_campaign_pkey PRIMARY KEY (campaign_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY petition_signature
    ADD CONSTRAINT petition_signature_pkey PRIMARY KEY (signature_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY private_message
    ADD CONSTRAINT private_message_pkey PRIMARY KEY (message_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY private_user_block
    ADD CONSTRAINT private_user_block__unique UNIQUE (user_id, blocked_user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY private_user_block
    ADD CONSTRAINT private_user_block_pkey PRIMARY KEY (block_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY profile
    ADD CONSTRAINT profile_pkey PRIMARY KEY (user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY simpletodo_list
    ADD CONSTRAINT simpletodo_list__unique UNIQUE (site_id, label);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY simpletodo_list
    ADD CONSTRAINT simpletodo_list_pkey PRIMARY KEY (list_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY site_backup
    ADD CONSTRAINT site_backup_pkey PRIMARY KEY (backup_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY site
    ADD CONSTRAINT site_pkey PRIMARY KEY (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY site_settings
    ADD CONSTRAINT site_settings_pkey PRIMARY KEY (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY site_super_settings
    ADD CONSTRAINT site_super_settings_pkey PRIMARY KEY (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY site_tag
    ADD CONSTRAINT site_tag__unique UNIQUE (site_id, tag);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY site_tag
    ADD CONSTRAINT site_tag_pkey PRIMARY KEY (tag_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY site_viewer
    ADD CONSTRAINT site_viewer_pkey PRIMARY KEY (viewer_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY storage_item
    ADD CONSTRAINT storage_item_pkey PRIMARY KEY (item_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY theme
    ADD CONSTRAINT theme_pkey PRIMARY KEY (theme_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY theme_preview
    ADD CONSTRAINT theme_preview_pkey PRIMARY KEY (theme_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ucookie
    ADD CONSTRAINT ucookie_pkey PRIMARY KEY (ucookie_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY user_abuse_flag
    ADD CONSTRAINT user_abuse_flag_pkey PRIMARY KEY (flag_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY user_block
    ADD CONSTRAINT user_block__unique UNIQUE (site_id, user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY user_block
    ADD CONSTRAINT user_block_pkey PRIMARY KEY (block_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY user_karma
    ADD CONSTRAINT user_karma_pkey PRIMARY KEY (user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY user_settings
    ADD CONSTRAINT user_settings_pkey PRIMARY KEY (user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY watched_page
    ADD CONSTRAINT wached_page__unique UNIQUE (user_id, page_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY watched_forum_thread
    ADD CONSTRAINT watched_forum_thread__unique UNIQUE (user_id, thread_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY watched_forum_thread
    ADD CONSTRAINT watched_forum_thread_pkey PRIMARY KEY (watched_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY watched_page
    ADD CONSTRAINT watched_page_pkey PRIMARY KEY (watched_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX admin_notification__site_id__idx ON admin_notification USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX anonymous_abuse_flag__address__idx ON anonymous_abuse_flag USING btree (address);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX anonymous_abuse_flag__site_id__idx ON anonymous_abuse_flag USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX category__name__idx ON category USING btree (name);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX category__site_id__idx ON category USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX email_invitation__site_id ON email_invitation USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX email_invitation__user_id ON email_invitation USING btree (user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX file__page_id__idx ON file USING btree (page_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX file__site_id__idx ON file USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX fki_forum_category__forum_post ON forum_category USING btree (last_post_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX forum_category__group_id__idx ON forum_category USING btree (group_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX forum_category__site_id__idx ON forum_category USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX forum_group__site_id__idx ON forum_group USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX forum_post__site_id__idx ON forum_post USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX forum_post__thread_id__idx ON forum_post USING btree (thread_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX forum_post__user_id__idx ON forum_post USING btree (user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX forum_post_revision__post_id__idx ON forum_post_revision USING btree (post_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX forum_thread__category_id__idx ON forum_thread USING btree (category_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX forum_thread__last_post_id__idx ON forum_thread USING btree (last_post_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX forum_thread__page_id__idx ON forum_thread USING btree (page_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX forum_thread__site_id__idx ON forum_thread USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX forum_thread__user_id__idx ON forum_thread USING btree (user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX front_forum_feed__site_id__idx ON front_forum_feed USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX fts_entry__forum_thread__idx ON fts_entry USING btree (thread_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX fts_entry__page_id__idx ON fts_entry USING btree (page_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX fts_entry__site_id__idx ON fts_entry USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX fts_entry__vector__idx ON fts_entry USING gist (vector);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX ip_block__ip__idx ON ip_block USING btree (ip);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX ip_block__site_id__idx ON ip_block USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX log_event__site_id__idx ON log_event USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX log_event__type__idx ON log_event USING btree (type);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE UNIQUE INDEX member__site_id_user_id__idx ON member USING btree (site_id, user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX member_application__site_id__idx ON member_application USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX member_application__user_id__idx ON member_application USING btree (user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX member_invitation__site_id__idx ON member_invitation USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX member_invitation__user_id__idx ON member_invitation USING btree (user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX moderator__site_id__idx ON moderator USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX moderator__user_id__idx ON moderator USING btree (user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX notification__user_id__idx ON notification USING btree (user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX ozone_session__user_id__idx ON ozone_session USING btree (user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE UNIQUE INDEX ozone_user__name__idx ON ozone_user USING btree (name);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE UNIQUE INDEX ozone_user__nick_name__idx ON ozone_user USING btree (nick_name);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE UNIQUE INDEX ozone_user__unix_name__idx ON ozone_user USING btree (unix_name);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX page__category_id__idx ON page USING btree (category_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX page__parent_page_id ON page USING btree (parent_page_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX page__revision_id__idx ON page USING btree (revision_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX page__site_id__idx ON page USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX page__unix_name__idx ON page USING btree (unix_name);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX page_abuse_flag__site_id__idx ON page_abuse_flag USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX page_edit_lock__page_id__idx ON page_edit_lock USING btree (page_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX page_edit_lock__site_id_page_unix_name ON page_edit_lock USING btree (site_id, page_unix_name);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX page_edit_lock__user_id__idx ON page_edit_lock USING btree (user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX page_inclusion__site_id ON page_inclusion USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX page_link__site_id ON page_link USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX page_revision__page_id__idx ON page_revision USING btree (page_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX page_revision__site_id__idx ON page_revision USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX page_revision__user_id__idx ON page_revision USING btree (user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX page_tag__page_id__idx ON page_tag USING btree (page_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX page_tag__site_id__idx ON page_tag USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX private_message__from_user_id__idx ON private_message USING btree (from_user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX private_message__to_user_id__idx ON private_message USING btree (to_user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX ront_forum_feed__page_id__idx ON front_forum_feed USING btree (page_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX simpletodo_list__site_id__idx ON simpletodo_list USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX site__custom_domain__idx ON site USING btree (custom_domain);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE UNIQUE INDEX site__unix_name__idx ON site USING btree (unix_name);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX site__visible__private__idx ON site USING btree (visible, private);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX ucookie__session_id_idx ON ucookie USING btree (session_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX ucookie__site_id ON ucookie USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX user_abuse_flag__site_id__idx ON user_abuse_flag USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE INDEX user_block__site_id__idx ON user_block USING btree (site_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO ozone_user DO SELECT currval('ozone_user_user_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO ozone_group DO SELECT currval('ozone_group_group_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO ozone_permission DO SELECT currval('ozone_permission_permission_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO ozone_user_group_relation DO SELECT currval('ozone_user_group_relation_user_group_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO ozone_user_permission_modifier DO SELECT currval('ozone_user_permission_modifier_user_permission_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO ozone_group_permission_modifier DO SELECT currval('ozone_group_permission_modifier_group_permission_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO site DO SELECT currval('site_site_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO site_tag DO SELECT currval('site_tag_tag_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO category DO SELECT currval('category_category_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO page DO SELECT currval('page_page_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO page_revision DO SELECT currval('page_revision_revision_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO page_source DO SELECT currval('page_source_source_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO page_metadata DO SELECT currval('page_metadata_metadata_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO fts_entry DO SELECT currval('fts_entry_fts_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO file DO SELECT currval('file_file_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO files_event DO SELECT currval('files_event_file_event_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO page_link DO SELECT currval('page_link_link_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO page_inclusion DO SELECT currval('page_inclusion_inclusion_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO member DO SELECT currval('member_member_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO admin DO SELECT currval('admin_admin_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO moderator DO SELECT currval('moderator_moderator_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO member_application DO SELECT currval('member_application_application_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO member_invitation DO SELECT currval('member_invitation_invitation_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO page_edit_lock DO SELECT currval('page_edit_lock_lock_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO theme DO SELECT currval('theme_theme_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO license DO SELECT currval('license_license_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO notification DO SELECT currval('notification_notification_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO private_message DO SELECT currval('private_message_message_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO global_ip_block DO SELECT currval('global_ip_block_block_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO ip_block DO SELECT currval('ip_block_block_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO global_user_block DO SELECT currval('global_user_block_block_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO user_block DO SELECT currval('user_block_block_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO private_user_block DO SELECT currval('private_user_block_block_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO watched_page DO SELECT currval('watched_page_watched_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO watched_forum_thread DO SELECT currval('watched_forum_thread_watched_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO page_abuse_flag DO SELECT currval('page_abuse_flag_flag_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO user_abuse_flag DO SELECT currval('user_abuse_flag_flag_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO anonymous_abuse_flag DO SELECT currval('anonymous_abuse_flag_flag_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO admin_notification DO SELECT currval('admin_notification_notification_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO forum_group DO SELECT currval('forum_group_group_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO forum_category DO SELECT currval('forum_category_category_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO forum_thread DO SELECT currval('forum_thread_thread_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO forum_post DO SELECT currval('forum_post_post_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO forum_post_revision DO SELECT currval('forum_post_revision_revision_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO front_forum_feed DO SELECT currval('front_forum_feed_feed_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO contact DO SELECT currval('contact_contact_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO page_rate_vote DO SELECT currval('page_rate_vote_rate_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO email_invitation DO SELECT currval('email_invitation_invitation_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO site_backup DO SELECT currval('site_backup_backup_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO domain_redirect DO SELECT currval('domain_redirect_redirect_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO site_viewer DO SELECT currval('site_viewer_viewer_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO openid_entry DO SELECT currval('openid_entry_openid_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO membership_link DO SELECT currval('membership_link_link_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO petition_campaign DO SELECT currval('petition_campaign_campaign_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO petition_signature DO SELECT currval('petition_signature_signature_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO simpletodo_list DO SELECT currval('simpletodo_list_list_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO comment DO SELECT currval('comment_comment_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO comment_revision DO SELECT currval('comment_revision_revision_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



CREATE RULE get_pkey_on_insert AS ON INSERT TO page_external_link DO SELECT currval('page_external_link_link_id_seq'::regclass) AS id;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY admin
    ADD CONSTRAINT admin__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY admin
    ADD CONSTRAINT admin__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY admin_notification
    ADD CONSTRAINT admin_notification__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY anonymous_abuse_flag
    ADD CONSTRAINT anonymous_abuse_flag__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY anonymous_abuse_flag
    ADD CONSTRAINT anonymous_abuse_flag__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY category
    ADD CONSTRAINT category__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY contact
    ADD CONSTRAINT contact__ozone_user__tagret_user_id FOREIGN KEY (target_user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY contact
    ADD CONSTRAINT contact__ozone_user__user_id FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY domain_redirect
    ADD CONSTRAINT domain_redirect__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY email_invitation
    ADD CONSTRAINT email_inviation__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY email_invitation
    ADD CONSTRAINT email_invitation__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY file
    ADD CONSTRAINT file__page FOREIGN KEY (page_id) REFERENCES page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY file
    ADD CONSTRAINT file__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE SET NULL;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY file
    ADD CONSTRAINT file__user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE SET NULL;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_category
    ADD CONSTRAINT forum_category__forum_group FOREIGN KEY (group_id) REFERENCES forum_group(group_id) ON UPDATE CASCADE ON DELETE RESTRICT;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_category
    ADD CONSTRAINT forum_category__forum_post FOREIGN KEY (last_post_id) REFERENCES forum_post(post_id) ON UPDATE CASCADE ON DELETE SET NULL;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_category
    ADD CONSTRAINT forum_category__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_group
    ADD CONSTRAINT forum_group__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_post
    ADD CONSTRAINT forum_post__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_post
    ADD CONSTRAINT forum_post__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_settings
    ADD CONSTRAINT forum_settings__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_thread
    ADD CONSTRAINT forum_thread__forum_category FOREIGN KEY (category_id) REFERENCES forum_category(category_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_thread
    ADD CONSTRAINT forum_thread__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE SET NULL;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_thread
    ADD CONSTRAINT forum_thread__page FOREIGN KEY (page_id) REFERENCES page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_thread
    ADD CONSTRAINT forum_thread__post FOREIGN KEY (last_post_id) REFERENCES forum_post(post_id) ON UPDATE CASCADE ON DELETE SET NULL;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY forum_thread
    ADD CONSTRAINT forum_thread__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY front_forum_feed
    ADD CONSTRAINT front_forum_feed__page FOREIGN KEY (page_id) REFERENCES page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY front_forum_feed
    ADD CONSTRAINT front_forum_feed__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY fts_entry
    ADD CONSTRAINT fts_entry__forum_thread FOREIGN KEY (thread_id) REFERENCES forum_thread(thread_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY fts_entry
    ADD CONSTRAINT fts_entry__page FOREIGN KEY (page_id) REFERENCES page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY fts_entry
    ADD CONSTRAINT fts_entry__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ip_block
    ADD CONSTRAINT ip_block__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY log_event
    ADD CONSTRAINT log_event__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE SET NULL;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY member
    ADD CONSTRAINT member__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY member
    ADD CONSTRAINT member__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY member_application
    ADD CONSTRAINT member_application__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY member_application
    ADD CONSTRAINT member_application__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY member_invitation
    ADD CONSTRAINT member_invitation__ozone_user__by_user_id FOREIGN KEY (by_user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY member_invitation
    ADD CONSTRAINT member_invitation__ozone_user__user_id FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY member_invitation
    ADD CONSTRAINT member_invitation__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY moderator
    ADD CONSTRAINT moderator__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY moderator
    ADD CONSTRAINT moderator__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY notification
    ADD CONSTRAINT notification__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ozone_session
    ADD CONSTRAINT ozone_session__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page
    ADD CONSTRAINT page__parent_page FOREIGN KEY (parent_page_id) REFERENCES page(page_id) ON UPDATE CASCADE ON DELETE SET NULL;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page
    ADD CONSTRAINT page__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_abuse_flag
    ADD CONSTRAINT page_abuse_flag__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_abuse_flag
    ADD CONSTRAINT page_abuse_flag__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_compiled
    ADD CONSTRAINT page_compiled__page FOREIGN KEY (page_id) REFERENCES page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_edit_lock
    ADD CONSTRAINT page_edit_lock__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_edit_lock
    ADD CONSTRAINT page_edit_lock__page FOREIGN KEY (page_id) REFERENCES page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_inclusion
    ADD CONSTRAINT page_inclusion__page__included_page_id FOREIGN KEY (included_page_id) REFERENCES page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_inclusion
    ADD CONSTRAINT page_inclusion__page__including_page_id FOREIGN KEY (including_page_id) REFERENCES page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_link
    ADD CONSTRAINT page_link__page__from_page_id FOREIGN KEY (from_page_id) REFERENCES page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_link
    ADD CONSTRAINT page_link__page__to_page_id FOREIGN KEY (to_page_id) REFERENCES page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_rate_vote
    ADD CONSTRAINT page_rate_vote__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_rate_vote
    ADD CONSTRAINT page_rate_vote__page FOREIGN KEY (page_id) REFERENCES page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY page_tag
    ADD CONSTRAINT page_tag__page FOREIGN KEY (page_id) REFERENCES page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY site_viewer
    ADD CONSTRAINT page_viewer__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY private_message
    ADD CONSTRAINT private_message__ozone_user__from_user_id FOREIGN KEY (from_user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY private_message
    ADD CONSTRAINT private_message__ozone_user__to_user_id FOREIGN KEY (to_user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY private_user_block
    ADD CONSTRAINT private_user_block__ozone_user__blocked_user_id FOREIGN KEY (blocked_user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY private_user_block
    ADD CONSTRAINT private_user_block__ozone_user__user_id FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY profile
    ADD CONSTRAINT profile__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY simpletodo_list
    ADD CONSTRAINT simpletedo_list__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY site_backup
    ADD CONSTRAINT site_backup__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY site_settings
    ADD CONSTRAINT site_settings__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY site_super_settings
    ADD CONSTRAINT site_super_settings__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY site_tag
    ADD CONSTRAINT site_tag__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY site_viewer
    ADD CONSTRAINT site_viewer__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY theme
    ADD CONSTRAINT theme__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ucookie
    ADD CONSTRAINT ucookie__ozone_session FOREIGN KEY (session_id) REFERENCES ozone_session(session_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY ucookie
    ADD CONSTRAINT ucookie__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY user_abuse_flag
    ADD CONSTRAINT user_abuse_flag__ozone_user__target_user_id FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY user_abuse_flag
    ADD CONSTRAINT user_abuse_flag__ozone_user__user_id FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY user_abuse_flag
    ADD CONSTRAINT user_abuse_flag__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY user_block
    ADD CONSTRAINT user_block__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY user_block
    ADD CONSTRAINT user_block__site FOREIGN KEY (site_id) REFERENCES site(site_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY user_settings
    ADD CONSTRAINT user_settings__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY watched_forum_thread
    ADD CONSTRAINT wached_forum_thread__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY watched_forum_thread
    ADD CONSTRAINT watched_forum_thread__forum_thread FOREIGN KEY (thread_id) REFERENCES forum_thread(thread_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY watched_page
    ADD CONSTRAINT watched_page__ozone_user FOREIGN KEY (user_id) REFERENCES ozone_user(user_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



ALTER TABLE ONLY watched_page
    ADD CONSTRAINT watched_page__page FOREIGN KEY (page_id) REFERENCES page(page_id) ON UPDATE CASCADE ON DELETE CASCADE;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;



