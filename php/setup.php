<?php
if (!defined('WIKIDOT_SETUP_COMPLETED')) {
    // assume that computer's clock runs in UTC
    putenv("TZ=UTC");
    if (function_exists('date_default_timezone_set')) {
        date_default_timezone_set('UTC');
    }

    // add settings for error-reporting
    error_reporting(E_ALL&~E_NOTICE); // hardcode ;-)

    // determine WIKIDOT_ROOT directory
    if (!defined('WIKIDOT_ROOT')) {
        define('WIKIDOT_ROOT', dirname(dirname(__FILE__)));
        define('OZONE_ROOT', WIKIDOT_ROOT.DIRECTORY_SEPARATOR.'vendor'.DIRECTORY_SEPARATOR.'scpwiki'.DIRECTORY_SEPARATOR.'ozoneframework');
    }

    require_once(WIKIDOT_ROOT.DIRECTORY_SEPARATOR."php/utils/GlobalProperties.php");
    require_once(WIKIDOT_ROOT.DIRECTORY_SEPARATOR."vendor/autoload.php");
    require_once(WIKIDOT_ROOT.DIRECTORY_SEPARATOR."vendor/scpwiki/ozoneframework/php/core/functions.php");
    require_once(WIKIDOT_ROOT.DIRECTORY_SEPARATOR."vendor/scpwiki/ozoneframework/php/core/autoload.inc.php");

    if (! GlobalProperties::$WIKI_FARM) {
        $_SERVER['HTTP_HOST'] = GlobalProperties::$URL_HOST;
        GlobalProperties::$SESSION_COOKIE_DOMAIN = null;
    }

    define('WIKIDOT_SETUP_COMPLETED', true);
}
