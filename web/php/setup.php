<?php

use Wikidot\Utils\GlobalProperties;

if (!defined('WIKIJUMP_SETUP_COMPLETED')) {
    // assume that computer's clock runs in UTC
    putenv("TZ=UTC");
    if (function_exists('date_default_timezone_set')) {
        date_default_timezone_set('UTC');
    }

    // add settings for error-reporting
    error_reporting(E_ALL&~E_NOTICE); // hardcode

    // determine WIKIJUMP_ROOT directory
    if (!defined('WIKIJUMP_ROOT')) {
        define('WIKIJUMP_ROOT', dirname(dirname(__FILE__)));
        define('OZONE_ROOT', WIKIJUMP_ROOT.DIRECTORY_SEPARATOR.'lib'.DIRECTORY_SEPARATOR.'ozoneframework');
    }

    require_once(WIKIJUMP_ROOT.DIRECTORY_SEPARATOR."php/Utils/GlobalProperties.php");
    require_once(WIKIJUMP_ROOT.DIRECTORY_SEPARATOR."vendor/autoload.php");
    require_once(WIKIJUMP_ROOT.DIRECTORY_SEPARATOR."lib/ozoneframework/php/core/functions.php");
    require_once(WIKIJUMP_ROOT.DIRECTORY_SEPARATOR."lib/ozoneframework/php/core/autoload.inc.php");

    if (! GlobalProperties::$WIKI_FARM) {
        $_SERVER['HTTP_HOST'] = GlobalProperties::$URL_HOST;
        GlobalProperties::$SESSION_COOKIE_DOMAIN = null;
    }

    define('WIKIJUMP_SETUP_COMPLETED', true);
}
