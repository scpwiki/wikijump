-- This table adds Wikidot's legacy tables, but with the "wikidot__" prefix so they are namespaced.
--
-- They are not intended to be used, and should be dropped in subsequent migrations as their
-- functionality is replaced or implemented.

CREATE TABLE wikidot__admin (
    admin_id SERIAL PRIMARY KEY,
    site_id INT,
    user_id INT,
    founder BOOLEAN DEFAULT false,

    UNIQUE (site_id, user_id)
);

CREATE TABLE wikidot__admin_notification (
    notification_id SERIAL PRIMARY KEY,
    site_id INT,
    body TEXT,
    type TEXT,
    viewed BOOLEAN,
    date TIMESTAMP,
    extra BYTEA,
    notify_online BOOLEAN DEFAULT false,
    notify_feed BOOLEAN DEFAULT false,
    notify_email BOOLEAN DEFAULT false
);

CREATE TABLE wikidot__anonymous_abuse_flag (
    flag_id SERIAL PRIMARY KEY,
    user_id INT,
    address INET,
    proxy BOOLEAN DEFAULT false,
    site_id INT,
    site_valid BOOLEAN DEFAULT true,
    global_valid BOOLEAN DEFAULT true
);

CREATE TABLE wikidot__category (
    category_id SERIAL PRIMARY KEY,
    site_id INT,
    name TEXT,
    theme_default BOOLEAN NOT NULL DEFAULT true,
    theme_id INT,
    permissions_default BOOLEAN NOT NULL DEFAULT true,
    permissions TEXT,
    license_default BOOLEAN NOT NULL DEFAULT true,
    license_id INT,
    license_other TEXT,
    nav_default BOOLEAN NOT NULL DEFAULT true,
    top_bar_page_name TEXT,
    side_bar_page_name TEXT,
    template_id INT,
    per_page_discussion BOOLEAN,
    per_page_discussion_default BOOLEAN DEFAULT true,
    rating TEXT,
    category_template_id INT,
    theme_external_url TEXT,
    autonumerate BOOLEAN NOT NULL DEFAULT false,
    page_title_template TEXT
);

CREATE TABLE wikidot__category_template (
    category_template_id SERIAL PRIMARY KEY,
    source TEXT
);

CREATE TABLE wikidot__comment (
    comment_id SERIAL PRIMARY KEY,
    page_id INT,
    parent_id INT,
    user_id INT,
    user_string TEXT,
    title TEXT,
    text TEXT,
    date_posted TIMESTAMP,
    site_id INT,
    revision_number INT,
    revision_id INT,
    date_last_edited TIMESTAMP,
    edited_user_id INT,
    edited_user_string TEXT
);

CREATE TABLE wikidot__comment_revision (
    revision_id SERIAL PRIMARY KEY,
    comment_id INT,
    user_id INT,
    user_string TEXT,
    text TEXT,
    title TEXT,
    date TIMESTAMP
);

CREATE TABLE wikidot__contact (
    contact_id SERIAL PRIMARY KEY,
    user_id INT,
    target_user_id INT,

    UNIQUE (user_id, target_user_id)
);

CREATE TABLE wikidot__domain_redirect (
    redirect SERIAL PRIMARY KEY,
    site_id INT,
    url TEXT,

    UNIQUE (site_id, url)
);

CREATE TABLE wikidot__email_invitations (
    invitation_id SERIAL PRIMARY KEY,
    hash TEXT,
    email TEXT,
    name TEXT,
    user_id INT,
    site_id INT,
    become_member BOOLEAN DEFAULT true,
    to_contacts BOOLEAN,
    message TEXT,
    attempts INT DEFAULT 1,
    accepted BOOLEAN NOT NULL DEFAULT false,
    delivered BOOLEAN NOT NULL DEFAULT true,
    date TIMESTAMP
);

CREATE TABLE wikidot__file (
    file_id SERIAL PRIMARY KEY,
    page_id INT,
    site_id INT,
    filename TEXT,
    mimetype TEXT,
    description TEXT,
    description_short TEXT,
    comment TEXT,
    size INT,
    date_added TIMESTAMP,
    user_id INT,
    user_string TEXT,
    has_resized BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE wikidot__files_event (
    file_event_id SERIAL PRIMARY KEY,
    filename TEXT,
    date TIMESTAMP,
    user_id INT,
    user_string TEXT,
    action TEXT,
    action_extra TEXT
);

CREATE TABLE wikidot__forum_category (
    category_id SERIAL PRIMARY KEY,
    group_id INT,
    name TEXT,
    description TEXT,
    number_posts INT NOT NULL DEFAULT 0,
    number_threads INT NOT NULL DEFAULT 0,
    last_post_id INT,
    permissions_default BOOLEAN NOT NULL DEFAULT true,
    permissions TEXT,
    max_nest_level INT,
    sort_index INT NOT NULL DEFAULT 0,
    site_id INT,
    per_page_discussion BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE wikidot__forum_group (
    group_id SERIAL PRIMARY KEY,
    name TEXT,
    description TEXT,
    sort_index INT NOT NULL DEFAULT 0,
    site_id INT,
    visible BOOLEAN DEFAULT true
);

CREATE TABLE wikidot__forum_post (
    post_id SERIAL PRIMARY KEY,
    thread_id INT,
    parent_id INT,
    user_id INT,
    user_string TEXT,
    title TEXT,
    text TEXT,
    date_posted TIMESTAMP,
    site_id INT,
    revision_number INT NOT NULL DEFAULT 0,
    revision_id INT,
    date_last_edited TIMESTAMP,
    edited_user_id INT,
    edited_user_string TEXT
);

CREATE TABLE wikidot__forum_post_revision (
    revision_id SERIAL PRIMARY KEY,
    post_id INT,
    user_id INT,
    user_string TEXT,
    text TEXT,
    title TEXT,
    date TIMESTAMP
);

CREATE TABLE wikidot__forum_settings (
    site_id SERIAL PRIMARY KEY,
    permissions TEXT,
    per_page_discussion BOOLEAN NOT NULL DEFAULT false,
    max_nest_level INT NOT NULL DEFAULT 0
);

CREATE TABLE wikidot__forum_thread (
    thread_id SERIAL PRIMARY KEY,
    user_id INT,
    user_string TEXT,
    category_id INT,
    title TEXT,
    description TEXT,
    number_posts INT,
    date_started TIMESTAMP,
    site_id INT,
    last_post_id INT,
    page_id INT,
    sticky BOOLEAN NOT NULL DEFAULT false,
    blocked BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE wikidot__forum_feed (
    feed_id SERIAL PRIMARY KEY,
    page_id INT,
    title TEXT,
    label TEXT,
    description TEXT,
    categories TEXT,
    permhash TEXT,
    site_id INT
);

CREATE TABLE wikidot__global_ip_block (
    block_id SERIAL PRIMARY KEY,
    address INET,
    flag_proxy BOOLEAN NOT NULL DEFAULT false,
    reason TEXT,
    flag_total BOOLEAN NOT NULL DEFAULT false,
    date_blocked TIMESTAMP
);

CREATE TABLE wikidot__global_user_block (
    block_id SERIAL PRIMARY KEY,
    site_id INT,
    user_id INT,
    reason TEXT,
    date_blocked TIMESTAMP
);

CREATE TABLE wikidot__ip_block (
    block_id SERIAL PRIMARY KEY,
    site_id INT,
    ip INET,
    flag_proxy BOOLEAN NOT NULL DEFAULT false,
    reason TEXT,
    date_blocked TIMESTAMP
);

CREATE TABLE wikidot__license (
    license_id SERIAL PRIMARY KEY,
    name TEXT,
    description TEXT,
    sort INT NOT NULL DEFAULT 0
);

CREATE TABLE wikidot__log_event (
    event_id SERIAL PRIMARY KEY,
    date TIMESTAMP,
    user_id INT,
    ip INET,
    proxy INET,
    type TEXT,
    site_id INT,
    page_id INT,
    revision_id INT,
    thread_id INT,
    post_id INT,
    user_agent TEXT,
    text TEXT
);

CREATE TABLE wikidot__member (
    member_id SERIAL PRIMARY KEY,
    site_id INT,
    user_id INT,
    date_joined TIMESTAMP,
    allow_newsletter BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE wikidot__member_application (
    application_id SERIAL PRIMARY KEY,
    site_id INT,
    user_id INT,
    status TEXT NOT NULL DEFAULT 'pending',
    date TIMESTAMP,
    comment TEXT NOT NULL,
    reply TEXT NOT NULL
);

CREATE TABLE wikidot__invitation (
    invitation_id SERIAL PRIMARY KEY,
    site_id INT,
    user_id INT,
    by_user_id INT,
    date TIMESTAMP,
    body TEXT NOT NULL
);

CREATE TABLE wikidot__membership_link (
    link_id SERIAL PRIMARY KEY,
    site_id INT,
    by_user_id INT,
    user_id INT,
    date TIMESTAMP,
    type TEXT
);

CREATE TABLE wikidot__moderator (
    moderator_id SERIAL PRIMARY KEY,
    site_id INT,
    user_id INT,
    permissions TEXT,

    UNIQUE (site_id, user_id)
);

CREATE TABLE wikidot__notification (
    notification_id SERIAL PRIMARY KEY,
    user_id INT,
    body TEXT,
    type TEXT,
    viewed BOOLEAN NOT NULL DEFAULT false,
    date TIMESTAMP,
    extra BYTEA,
    notify_online BOOLEAN NOT NULL DEFAULT true,
    notify_feed BOOLEAN NOT NULL DEFAULT false,
    notify_email BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE wikidot__ozone_group (
    group_id SERIAL PRIMARY KEY,
    parent_group_id INT,
    name TEXT,
    description TEXT
);

CREATE TABLE wikidot__ozone_group_permission_modifier (
    group_permission_id SERIAL PRIMARY KEY,
    group_id TEXT,
    permission_id TEXT,
    modifier INT
);

CREATE TABLE wikidot__ozone_lock (
    key TEXT PRIMARY KEY
);

CREATE TABLE wikidot__ozone_permission (
    permission_id SERIAL PRIMARY KEY,
    name TEXT,
    description TEXT
);

CREATE TABLE wikidot__ozone_session (
    session_id TEXT PRIMARY KEY,
    started TIMESTAMP,
    last_accessed TIMESTAMP,
    ip_address TEXT,
    check_ip BOOLEAN NOT NULL DEFAULT false,
    infinite BOOLEAN NOT NULL DEFAULT false,
    user_id INT,
    serialized_datablock BYTEA,
    ip_address_ssl TEXT,
    ua_hash TEXT
);

CREATE TABLE wikidot__ozone_user_group_relation (
    user_group_id SERIAL PRIMARY KEY,
    user_id INT,
    group_id INT
);

CREATE TABLE wikidot__ozone_user_permission_modifier (
    user_permission_id SERIAL PRIMARY KEY,
    user_id INT,
    permission_id TEXT,
    modifier INT
);

CREATE TABLE wikidot__page (
    page_id SERIAL PRIMARY KEY,
    site_id INT,
    category_id INT,
    parent_page_id INT,
    revision_id INT,
    source_id INT,
    metadata_id INT,
    revision_number INT NOT NULL DEFAULT 0,
    title TEXT,
    unix_name TEXT,
    date_created TIMESTAMP,
    date_last_edited TIMESTAMP,
    last_edit_user_id INT,
    last_edit_user_string TEXT,
    thread_id INT,
    owner_user_id INT,
    blocked BOOLEAN NOT NULL DEFAULT false,
    rate INT NOT NULL DEFAULT 0
);

CREATE TABLE wikidot__page_abuse_flag (
    flag_id SERIAL PRIMARY KEY,
    user_id INT,
    site_id INT,
    path TEXT,
    site_valid BOOLEAN NOT NULL DEFAULT true,
    global_valid BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE wikidot__page_compiled (
    page_id SERIAL PRIMARY KEY,
    text TEXT,
    date_compiled TIMESTAMP
);

CREATE TABLE wikidot__page_edit_lock (
    lock_id SERIAL PRIMARY KEY,
    page_id INT,
    mode TEXT NOT NULL DEFAULT 'page',
    section_id INT,
    range_start INT,
    range_end INT,
    page_unix_name TEXT,
    user_id INT,
    user_string TEXT,
    session_id TEXT,
    date_started TIMESTAMP,
    date_last_accessed TIMESTAMP,
    secret TEXT,
    site_id INT,

    UNIQUE (site_id, page_unix_name)
);

CREATE TABLE wikidot__page_external_link (
    link_id SERIAL PRIMARY KEY,
    site_id INT,
    page_id INT,
    to_url TEXT,
    date TIMESTAMP
);

CREATE TABLE wikidot__page_inclusion (
    inclusion_id SERIAL PRIMARY KEY,
    including_page_id INT,
    included_page_id INT,
    included_page_name TEXT,
    site_id INT,

    UNIQUE (including_page_id, included_page_id, included_page_name)
);

CREATE TABLE wikidot__page_link (
    link_id SERIAL PRIMARY KEY,
    from_page_id INT,
    to_page_id INT,
    to_page_name TEXT,
    site_id INT,

    UNIQUE (from_page_id, to_page_id, to_page_name)
);

CREATE TABLE wikidot__page_metadata (
    metadata_id SERIAL PRIMARY KEY,
    parent_page_id INT,
    title TEXT,
    unix_name TEXT,
    owner_user_id INT
);

CREATE TABLE wikidot__page_rate_vote (
    rate_id SERIAL PRIMARY KEY,
    user_id INT,
    page_id INT,
    rate INT NOT NULL DEFAULT 1,
    date TIMESTAMP,

    UNIQUE (user_id, page_id)
);

CREATE TABLE wikidot__page_revision (
    revision_id SERIAL PRIMARY KEY,
    page_id INT,
    source_id INT,
    metadata_id INT,
    flags TEXT,
    flag_text BOOLEAN NOT NULL DEFAULT false,
    flag_title BOOLEAN NOT NULL DEFAULT false,
    flag_file BOOLEAN NOT NULL DEFAULT false,
    flag_rename BOOLEAN NOT NULL DEFAULT false,
    flag_meta BOOLEAN NOT NULL DEFAULT false,
    flag_new BOOLEAN NOT NULL DEFAULT false,
    since_full_source INT,
    diff_source BOOLEAN NOT NULL DEFAULT false,
    revision_number INT,
    date_last_edited TIMESTAMP,
    user_id INT,
    user_string TEXT,
    comments TEXT,
    flag_new_site BOOLEAN NOT NULL DEFAULT false,
    site_id INT
);

CREATE TABLE wikidot__page_source (
    source_id SERIAL PRIMARY KEY,
    text TEXT
);

CREATE TABLE wikidot__page_tag (
    tag_id SERIAL PRIMARY KEY,
    site_id INT,
    page_id INT,
    tag TEXT
);

CREATE TABLE wikidot__private_message (
    message_id SERIAL PRIMARY KEY,
    from_user_id INT,
    to_user_id INT,
    subject TEXT,
    body TEXT,
    date TIMESTAMP,
    flag BOOLEAN,
    flag_new BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE wikidot__private_user_block (
    block_id SERIAL PRIMARY KEY,
    user_id INT,
    blocked_user_id INT,

    UNIQUE (user_id, blocked_user_id)
);

CREATE TABLE wikidot__profile (
    user_id INT PRIMARY KEY,
    real_name TEXT,
    gender TEXT,
    birthday_day INT,
    birthday_month INT,
    birthday_year INT,
    about TEXT,
    location TEXT,
    website TEXT,
    im_icq TEXT,
    im_jabber TEXT,
    change_screen_name_count INT NOT NULL DEFAULT 0
);

CREATE TABLE wikidot__site (
    site_id SERIAL PRIMARY KEY,
    name TEXT,
    subtitle TEXT,
    unix_name TEXT,
    description TEXT,
    language TEXT NOT NULL DEFAULT 'en',
    date_created TIMESTAMP,
    custom_domain TEXT,
    visible BOOLEAN NOT NULL DEFAULT true,
    default_page TEXT NOT NULL DEFAULT 'start',
    private BOOLEAN NOT NULL DEFAULT false,
    deleted BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE wikidot__site_backup (
    backup_id SERIAL PRIMARY KEY,
    site_id INT,
    status TEXT,
    backup_source BOOLEAN NOT NULL DEFAULT true,
    backup_files BOOLEAN NOT NULL DEFAULT true,
    date TIMESTAMP,
    rand TEXT
);

CREATE TABLE wikidot__site_settings (
    site_id INT PRIMARY KEY,
    allow_membership_by_apply BOOLEAN NOT NULL DEFAULT true,
    allow_membership_by_password BOOLEAN NOT NULL DEFAULT false,
    membership_password TEXT,
    file_storage_size INT NOT NULL DEFAULT 314572800,
    use_ganalytics BOOLEAN NOT NULL DEFAULT false,
    private_landing_page TEXT NOT NULL DEFAULT 'system:join',
    max_private_members INT NOT NULL DEFAULT 50,
    max_private_viewers INT NOT NULL DEFAULT 20,
    hide_navigation_unauthorized BOOLEAN NOT NULL DEFAULT true,
    ssl_mode TEXT,
    allow_members_invite BOOLEAN NOT NULL DEFAULT false,
    max_upload_file_size INT NOT NULL DEFAULT 10485760
);

CREATE TABLE wikidot__site_super_settings (
    site_id INT PRIMARY KEY,
    can_custom_domain BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE wikidot__site_tag (
    tag_id SERIAL PRIMARY KEY,
    site_id INT,
    tag TEXT,

    UNIQUE (site_id, tag)
);

CREATE TABLE wikidot__site_viewer (
    viewer_id SERIAL PRIMARY KEY,
    site_id INT,
    user_id INT
);

CREATE TABLE wikidot__theme (
    theme_id SERIAL PRIMARY KEY,
    name TEXT,
    unix_name TEXT,
    abstract BOOLEAN NOT NULL DEFAULT false,
    extends_theme_id INT,
    variant_of_theme_id INT,
    custom BOOLEAN NOT NULL DEFAULT false,
    site_id INT,
    use_side_bar BOOLEAN NOT NULL DEFAULT true,
    use_top_bar BOOLEAN NOT NULL DEFAULT true,
    sort_index INT NOT NULL DEFAULT 0,
    sync_page_name TEXT,
    revision_number INT NOT NULL DEFAULT 0
);

CREATE TABLE wikidot__theme_preview (
    theme_id INT PRIMARY KEY,
    body TEXT
);

CREATE TABLE wikidot__ucookie (
    ucookie_id TEXT PRIMARY KEY,
    site_id INT,
    session_id TEXT,
    date_granted TIMESTAMP
);

CREATE TABLE wikidot__user_abuse_flag (
    flag_id SERIAL PRIMARY KEY,
    user_id INT,
    target_user_id INT,
    site_id INT,
    site_valid BOOLEAN NOT NULL DEFAULT true,
    global_valid BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE wikidot__user_block (
    block_id SERIAL PRIMARY KEY,
    site_id INT,
    user_id INT,
    reason TEXT,
    date_blocked TIMESTAMP,

    UNIQUE (site_id, user_id)
);

CREATE TABLE wikidot__user_karma (
    user_id INT PRIMARY KEY,
    points INT,
    level INT
);

CREATE TABLE wikidot__user_settings (
    user_id INT PRIMARY KEY,
    receive_invitations BOOLEAN NOT NULL DEFAULT true,
    receive_pm TEXT NOT NULL DEFAULT 'a',
    notify_online TEXT NOT NULL DEFAULT '*',
    notify_feed TEXT NOT NULL DEFAULT '*',
    notify_email TEXT NOT NULL DEFAULT '*',
    receive_newsletter BOOLEAN NOT NULL DEFAULT true,
    receive_digest BOOLEAN NOT NULL DEFAULT true,
    allow_site_newsletters_default BOOLEAN NOT NULL DEFAULT true,
    max_sites_admin INT NOT NULL DEFAULT 3
);

CREATE TABLE wikidot__watched_forum_thread (
    watched_id SERIAL PRIMARY KEY,
    user_id INT,
    thread_id INT,

    UNIQUE (user_id, thread_id)
);

CREATE TABLE wikidot__watched_page (
    watched_id SERIAL PRIMARY KEY,
    user_id INT,
    page_id INT,

    UNIQUE (user_id, page_id)
);
