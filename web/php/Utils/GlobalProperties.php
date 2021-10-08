<?php

namespace Wikidot\Utils;

use Exception;

/**
 * The Wikijump GlobalProperties Class is used to parse
 * the ini file and access any settings and properties
 *
 */
class GlobalProperties
{

    // main settings
    public static $SERVICE_NAME;
    public static $URL_DOMAIN;
    public static $URL_HOST;
    public static $WIKI_FARM;

    // security settings
    public static $ALLOW_ANY_HTTP;
    public static $USE_SSL;
    public static $HTTP_SCHEMA;
    public static $SECRET;
    public static $SECRET_DOMAIN_LOGIN;
    public static $USE_UPLOAD_DOMAIN;
    public static $URL_UPLOAD_DOMAIN;
    public static $RESTRICT_HTML;
    public static $SECRET_MANAGE_SUPERADMIN;
    public static $SECRET_LOGIN_SEED;

    // database settings
    public static $DATABASE_SERVER;
    public static $DATABASE_PORT;
    public static $DATABASE_USER;
    public static $DATABASE_PASSWORD;
    public static $DATABASE_NAME;

    // search settings
    public static $SEARCH_LUCENE_INDEX;
    public static $SEARCH_LUCENE_QUEUE;
    public static $SEARCH_LUCENE_LOCK;
    public static $SEARCH_HIGHLIGHT;
    public static $SEARCH_USE_JAVA;

    // mail settings
    public static $DEFAULT_SMTP_HOST;
    public static $DEFAULT_SMTP_PORT;
    public static $DEFAULT_SMTP_USER;
    public static $DEFAULT_SMTP_PASSWORD;
    public static $DEFAULT_SMTP_SECURE;
    public static $DEFAULT_SMTP_AUTH;
    public static $DEFAULT_SMTP_HOSTNAME;
    public static $DEFAULT_SMTP_FROM_EMAIL;
    public static $DEFAULT_SMTP_FROM_NAME;
    public static $DEFAULT_SMTP_REPLY_TO;
    public static $DEFAULT_SMTP_SENDER;
    public static $SUPPORT_EMAIL;

    // memcache settings
    public static $USE_MEMCACHE;
    public static $MEMCACHE_HOST;
    public static $MEMCACHE_PORT;

    // session settings
    public static $SESSION_TIMEOUT;
    public static $SESSION_COOKIE_NAME;
    public static $SESSION_COOKIE_NAME_SSL;
    public static $SESSION_COOKIE_SECURE;
    public static $SESSION_COOKIE_NAME_IE;

    // ui settings
    public static $UI_SLEEP;
    public static $DEFAULT_LANGUAGE;

    // log settings
    public static $LOGGER_LEVEL;
    public static $LOGGER_FILE;

    // feature flags
    public static $FEATURE_WIKITEXT_BACKEND;

    // other settings
    public static $CACHE_FILES_FOR;
    public static $URL_DOCS;
    public static $IP_HOST;
    public static $MODULES_JS_PATH;
    public static $MODULES_JS_URL;
    public static $MODULES_CSS_PATH;
    public static $MODULES_CSS_URL;

    // third-party keys
    public static $FR_CAPTCHA_SITE_KEY;
    public static $FR_CAPTCHA_API_KEY;

    // non-configurable properties
    public static $DATABASE_TYPE;
    public static $DATABASE_USE_PERSISTENT_CONNECTIONS;
    public static $SESSION_COOKIE_DOMAIN;
    public static $DEFAULT_SKIN;
    public static $URL_HOST_PREG;
    public static $URL_DOMAIN_PREG;
    public static $URL_UPLOAD_DOMAIN_PREG;

    /**
     * array with ini options processesed
     *
     * @var array
     */
    protected static $iniConfig;

    /**
     * get a configuration option from ini file
     * return a default one if none found
     * throw an exception if there is none found and no default supplied
     *
     * @param string $section
     * @param string $key
     * @param string $default
     * @return string
     */
    protected static function fromIni($section, $key, $default = null)
    {
        if (isset(self::$iniConfig[$section]) && isset(self::$iniConfig[$section][$key])) {
            $value = self::$iniConfig[$section][$key];
        } else {
            if ($default === null) {
                throw new Exception("You should set '$key' value in '$section' section in wikijump.ini file.");
            } else {
                $value = $default;
            }
        }
        return $value;
    }

    protected static function fromFile($file)
    {
        if ($fp = @fopen(WIKIJUMP_ROOT . '/conf/' . $file, 'r')) {
            $s = fread($fp, 4096);
            fclose($fp);
        } else {
            $s = "";
        }
        return trim($s);
    }

    /**
     * read wikijump.ini file
     * set some default values
     * calculate other values
     */
    public static function init()
    {

        self::$iniConfig = parse_ini_file(WIKIJUMP_ROOT . "/conf/wikijump.ini", true);

        // main settings
        self::$WIKI_FARM                = $_ENV["WIKIJUMP_WIKI_FARM"] ?? self::fromIni("main", "wiki_farm", true);

        if (self::$WIKI_FARM) {
            self::$SERVICE_NAME         = $_ENV["WIKIJUMP_SERVICE_NAME"] ?? self::fromIni("main", "service", "Wikijump");
            self::$URL_DOMAIN           = $_ENV["WIKIJUMP_URL_DOMAIN"] ?? self::fromIni("main", "domain", "wikijump.com");
            self::$URL_HOST             = $_ENV["WIKIJUMP_URL_HOST"] ?? self::fromIni("main", "main_wiki", "www." . self::$URL_DOMAIN);
        } else {
            self::$SERVICE_NAME         = $_ENV["WIKIJUMP_SERVICE_NAME"] ?? self::fromIni("main", "service", "Wikijump");
            self::$URL_DOMAIN           = $_ENV["WIKIJUMP_URL_DOMAIN"] ?? self::fromIni("main", "domain", "wikijump.com");
            self::$URL_HOST             = $_ENV["WIKIJUMP_URL_HOST"] ?? self::fromIni("main", "main_wiki", "www." . self::$URL_DOMAIN);
        }

        // security settings
        self::$SECRET                   = $_ENV["WIKIJUMP_SECRET"] ?? self::fromIni("security", "secret", md5('secret'));
        self::$ALLOW_ANY_HTTP           = $_ENV["WIKIJUMP_ALLOW_ANY_HTTP"] ?? self::fromIni("security", "allow_http", true);
        self::$USE_SSL                  = $_ENV["WIKIJUMP_USE_SSL"] ?? self::fromIni("security", "ssl", false);
        self::$HTTP_SCHEMA              = $_ENV["WIKIJUMP_HTTP_SCHEMA"] ?? self::fromIni("security", "schema", "https");
        self::$SECRET_DOMAIN_LOGIN      = $_ENV["WIKIJUMP_SECRET_DOMAIN_LOGIN"] ?? self::fromIni("security", "secret_login", self::$SECRET . "_custom_domain_login");
        self::$USE_UPLOAD_DOMAIN        = $_ENV["WIKIJUMP_USE_UPLOAD_DOMAIN"] ?? self::fromIni("security", "upload_separate_domain", true);
        self::$URL_UPLOAD_DOMAIN        = $_ENV["WIKIJUMP_URL_UPLOAD_DOMAIN"] ?? self::fromIni("security", "upload_domain", "files." . self::$URL_DOMAIN);
        self::$RESTRICT_HTML            = $_ENV["WIKIJUMP_RESTRICT_HTML"] ?? self::fromIni("security", "upload_restrict_html", true);
        self::$SECRET_MANAGE_SUPERADMIN = $_ENV["WIKIJUMP_SECRET_MANAGE_SUPERADMIN"] ?? self::fromIni("security", "secret_manage_superadmin", md5(self::$SECRET . '_super_admin'));
        self::$SECRET_LOGIN_SEED        = $_ENV["WIKIJUMP_SECRET_LOGIN_SEED"] ?? self::fromIni("security", "secret_login_seed", md5(self::$SECRET . '_login'));

        // database settings
        self::$DATABASE_USER            = $_ENV["WIKIJUMP_DATABASE_USER"] ?? self::fromIni("db", "user", "postgres");            // no default!
        self::$DATABASE_PASSWORD        = $_ENV["WIKIJUMP_DATABASE_PASSWORD"] ?? self::fromIni("db", "password", "postgres");        // no default!
        self::$DATABASE_NAME            = $_ENV["WIKIJUMP_DATABASE_NAME"] ?? self::fromIni("db", "database", "postgres");        // no default!
        self::$DATABASE_SERVER          = $_ENV["WIKIJUMP_DATABASE_SERVER"] ?? self::fromIni("db", "host", "127.0.0.1");
        self::$DATABASE_PORT            = $_ENV["WIKIJUMP_DATABASE_PORT"] ?? self::fromIni("db", "port", "5432");

        // search settings
        self::$SEARCH_LUCENE_INDEX      = $_ENV["WIKIJUMP_SEARCH_LUCENE_INDEX"] ?? self::fromIni("search", "lucene_index", WIKIJUMP_ROOT . "/tmp/lucene_index");
        self::$SEARCH_LUCENE_QUEUE      = $_ENV["WIKIJUMP_SEARCH_LUCENE_QUEUE"] ?? self::fromIni("search", "lucene_queue", WIKIJUMP_ROOT . "/tmp/lucene_queue");
        self::$SEARCH_LUCENE_LOCK       = $_ENV["WIKIJUMP_SEARCH_LUCENE_LOCK"] ?? self::fromIni("search", "lucene_lock", WIKIJUMP_ROOT . "/tmp/lucene_lock");
        self::$SEARCH_HIGHLIGHT         = $_ENV["WIKIJUMP_SEARCH_HIGHLIGHT"] ?? self::fromIni("search", "highlight", false);
        self::$SEARCH_USE_JAVA          = $_ENV["WIKIJUMP_SEARCH_USE_JAVA"] ?? self::fromIni("search", "use_java", false);

        // mail settings
        self::$DEFAULT_SMTP_HOST        = $_ENV["WIKIJUMP_DEFAULT_SMTP_HOST"] ?? self::fromIni("mail", "host", "127.0.0.1");
        self::$DEFAULT_SMTP_SECURE      = $_ENV["WIKIJUMP_DEFAULT_SMTP_SECURE"] ?? self::fromIni("mail", "ssl", false) ? "ssl" : "";
        self::$DEFAULT_SMTP_PORT        = $_ENV["WIKIJUMP_DEFAULT_SMTP_PORT"] ?? self::fromIni("mail", "port", (self::$DEFAULT_SMTP_SECURE == "ssl") ? 465 : 25);
        self::$DEFAULT_SMTP_USER        = $_ENV["WIKIJUMP_DEFAULT_SMTP_USER"] ?? self::fromIni("mail", "user", "admin");
        self::$DEFAULT_SMTP_PASSWORD    = $_ENV["WIKIJUMP_DEFAULT_SMTP_PASSWORD"] ?? self::fromIni("mail", "password", "password");
        self::$DEFAULT_SMTP_AUTH        = $_ENV["WIKIJUMP_DEFAULT_SMTP_AUTH"] ?? self::fromIni("mail", "auth", false);
        self::$DEFAULT_SMTP_HOSTNAME    = $_ENV["WIKIJUMP_DEFAULT_SMTP_HOSTNAME"] ?? self::fromIni("mail", "hostname", "mail" . self::$URL_DOMAIN);
        self::$DEFAULT_SMTP_FROM_EMAIL  = $_ENV["WIKIJUMP_DEFAULT_SMTP_FROM_EMAIL"] ?? self::fromIni("mail", "from_mail", "no-reply@" . self::$DEFAULT_SMTP_HOSTNAME);
        self::$DEFAULT_SMTP_FROM_NAME   = $_ENV["WIKIJUMP_DEFAULT_SMTP_FROM_NAME"] ?? self::fromIni("mail", "from_name", self::$SERVICE_NAME);
        self::$DEFAULT_SMTP_REPLY_TO    = $_ENV["WIKIJUMP_DEFAULT_SMTP_REPLY_TO"] ?? self::fromIni("mail", "reply_to", self::$DEFAULT_SMTP_FROM_EMAIL);
        self::$DEFAULT_SMTP_SENDER      = $_ENV["WIKIJUMP_DEFAULT_SMTP_SENDER"] ?? self::fromIni("mail", "sender", self::$DEFAULT_SMTP_FROM_EMAIL);
        self::$SUPPORT_EMAIL            = $_ENV["WIKIJUMP_SUPPORT_EMAIL"] ?? self::fromIni("mail", "support", "support@" . self::$DEFAULT_SMTP_HOSTNAME);

        // memcache settings
        self::$USE_MEMCACHE             = $_ENV["WIKIJUMP_USE_MEMCACHE"] ?? self::fromIni("memcached", "enable", true);
        self::$MEMCACHE_HOST            = $_ENV["WIKIJUMP_MEMCACHE_HOST"] ?? self::fromIni("memcached", "host", "127.0.0.1");
        self::$MEMCACHE_PORT            = $_ENV["WIKIJUMP_MEMCACHE_PORT"] ?? self::fromIni("memcached", "port", 11211);

        // session settings
        self::$SESSION_TIMEOUT          = $_ENV["WIKIJUMP_SESSION_TIMEOUT"] ?? self::fromIni("session", "timeout", 3600);
        self::$SESSION_COOKIE_NAME      = $_ENV["WIKIJUMP_SESSION_COOKIE_NAME"] ?? self::fromIni("session", "cookie_name", "WIKIJUMP_SESSION_ID");
        self::$SESSION_COOKIE_NAME_SSL  = $_ENV["WIKIJUMP_SESSION_COOKIE_NAME_SSL"] ?? self::fromIni("session", "cookie_name_ssl", self::$SESSION_COOKIE_NAME."_SECURE");
        self::$SESSION_COOKIE_SECURE    = $_ENV["WIKIJUMP_SESSION_COOKIE_SECURE"] ?? self::fromIni("session", "cookie_ssl", true);
        self::$SESSION_COOKIE_NAME_IE   = $_ENV["WIKIJUMP_SESSION_COOKIE_NAME_IE"] ?? self::fromIni("session", "ie_cookie_name", self::$SESSION_COOKIE_NAME . "_IE");

        // ui settings
        self::$UI_SLEEP                 = $_ENV["WIKIJUMP_UI_SLEEP"] ?? self::fromIni("ui", "sleep", true);
        self::$DEFAULT_LANGUAGE         = $_ENV["WIKIJUMP_DEFAULT_LANGUAGE"] ?? self::fromIni("ui", "language", "en");

        // log settings
        self::$LOGGER_LEVEL             = $_ENV["WIKIJUMP_LOGGER_LEVEL"] ?? self::fromIni("log", "level", "error");
        self::$LOGGER_FILE              = $_ENV["WIKIJUMP_LOGGER_FILE"] ?? self::fromIni("log", "file", "wikijump.log"); // TODO: use this setting

        // feature flags
        self::$FEATURE_WIKITEXT_BACKEND = $_ENV["FEATURE_WIKITEXT_BACKEND"] ?? self::fromIni("feature", "wikitext_backend", "text_wiki");

        // other settings
        self::$CACHE_FILES_FOR          = $_ENV["WIKIJUMP_CACHE_FILES_FOR"] ?? self::fromIni("misc", "cache_files_for", 0);
        self::$URL_DOCS                 = $_ENV["WIKIJUMP_URL_DOCS"] ?? self::fromIni("misc", "doc_url", self::$HTTP_SCHEMA. "://" . self::$URL_HOST . "/doc");
        self::$IP_HOST                  = $_ENV["WIKIJUMP_IP_HOST"] ?? self::fromIni("misc", "ip", "127.0.0.1");
        self::$MODULES_JS_PATH          = $_ENV["WIKIJUMP_MODULES_JS_PATH"] ?? self::fromIni("misc", "modules_js_path", "web/files--common/modules/js");
        self::$MODULES_JS_URL           = $_ENV["WIKIJUMP_MODULES_JS_URL"] ?? self::fromIni("misc", "modules_js_url", "/common--modules/js");
        self::$MODULES_CSS_PATH         = $_ENV["WIKIJUMP_MODULES_CSS_PATH"] ?? self::fromIni("misc", "modules_css_path", "web/files--common/modules/css");
        self::$MODULES_CSS_URL          = $_ENV["WIKIJUMP_MODULES_CSS_URL"] ?? self::fromIni("misc", "modules_css_url", "/common--modules/css");

        self::$FR_CAPTCHA_SITE_KEY      = $_ENV["WIKIJUMP_FR_CAPTCHA_SITE_KEY"] ?? self::fromIni("keys", "friendlycaptcha-site-key", "");
        self::$FR_CAPTCHA_API_KEY       = $_ENV["WIKIJUMP_FR_CAPTCHA_API_KEY"] ?? self::fromIni("keys", "friendlycaptcha-api-key", "");

        // non-configurable properties
        self::$DATABASE_TYPE            = "pgsql";
        self::$DATABASE_USE_PERSISTENT_CONNECTIONS = false;
        self::$SESSION_COOKIE_DOMAIN    = "." . self::$URL_DOMAIN;
        self::$DEFAULT_SKIN             = "default";
        self::$URL_HOST_PREG            = preg_quote(self::$URL_HOST);
        self::$URL_DOMAIN_PREG          = preg_quote(self::$URL_DOMAIN);
        self::$URL_UPLOAD_DOMAIN_PREG   = preg_quote(self::$URL_UPLOAD_DOMAIN);
    }
}

GlobalProperties::init();
