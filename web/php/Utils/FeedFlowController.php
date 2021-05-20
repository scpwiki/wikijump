<?php

namespace Wikidot\Utils;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Ozone\Framework\OzoneLogger;
use Ozone\Framework\OzoneLoggerFileOutput;
use Ozone\Framework\RunData;
use Ozone\Framework\SecurityManager;
use Ozone\Framework\WebFlowController;
use Wikidot\DB\SitePeer;
use Wikidot\DB\MemberPeer;
use Wikidot\DB\SiteViewerPeer;
use Wikijump\Helpers\LegacyTools;

class FeedFlowController extends WebFlowController
{

    public function process()
    {

        // initialize logging service
        $logger = OzoneLogger::instance();
        $loggerFileOutput = new OzoneLoggerFileOutput();
        $loggerFileOutput->setLogFileName(WIKIJUMP_ROOT."/logs/ozone.log");
        $logger->addLoggerOutput($loggerFileOutput);
        $logger->setDebugLevel(GlobalProperties::$LOGGER_LEVEL);

        $logger->debug("Feed request processing started, logger initialized");

        Ozone ::init();

        $runData = new RunData();
        $runData->init();
        Ozone :: setRunData($runData);
        $logger->debug("RunData object created and initialized");

        // check if site (Wiki) exists!
        $siteHost = $_SERVER["HTTP_HOST"];

        $memcache = Ozone::$memcache;
        if (preg_match("/^([a-zA-Z0-9\-]+)\." . GlobalProperties::$URL_DOMAIN . "$/", $siteHost, $matches)==1) {
            $siteUnixName=$matches[1];
            // select site based on the unix name

            // check memcached first!

            // the memcache block is to avoid database connection if possible

            $mcKey = 'site..'.$siteUnixName;
            $site = $memcache->get($mcKey);
            if ($site == false) {
                $c = new Criteria();
                $c->add("unix_name", $siteUnixName);
                $c->add("site.deleted", false);
                $site = SitePeer::instance()->selectOne($c);
                $memcache->set($mcKey, $site, 0, 3600);
            }
        } else {
            // select site based on the custom domain
            $mcKey = 'site_cd..'.$siteHost;
            $site = $memcache->get($mcKey);
            if ($site == false) {
                $c = new Criteria();
                $c->add("custom_domain", $siteHost);
                $c->add("site.deleted", false);
                $site = SitePeer::instance()->selectOne($c);
                $memcache->set($mcKey, $site, 0, 3600);
            }
            GlobalProperties::$SESSION_COOKIE_DOMAIN = '.'.$siteHost;
        }

        if ($site == null) {
            $content = file_get_contents(WIKIJUMP_ROOT."/resources/views/site_not_exists.html");
            echo $content;
            return $content;
        }

        $runData->setTemp("site", $site);
        //nasty global thing...
        $GLOBALS['siteId'] = $site->getSiteId();
        $GLOBALS['site'] = $site;

        // set language
        $lang = $site->getLanguage();
        $runData->setLanguage($lang);
        $GLOBALS['lang'] = $lang;

        // and for gettext too:

        switch ($lang) {
            case 'pl':
                $glang="pl_PL";
                break;
            case 'en':
                $glang="en_US";
                break;
        }

        putenv("LANG=$glang");
        putenv("LANGUAGE=$glang");
        setlocale(LC_ALL, $glang.'.UTF-8');

        $settings = $site->getSettings();
        // handle SSL
        $sslMode = $settings->getSslMode();
        if (isset($_SERVER['HTTPS'])) {
            if (!$sslMode) {
                // not enabled, redirect to http:
                echo _("Secure access is not enabled for this Wiki.");
                exit;
            }
        }

        $template = $runData->getScreenTemplate();
        $classFile = $runData->getScreenClassPath();
        $class = LegacyTools::getNamespacedClassFromPath($runData->getScreenClassPath());
        $logger->debug("processing template: ".$runData->getScreenTemplate().", Class: $class");

        require_once($classFile);
        $screen = new $class();

        // check if requires authentication
        if ($screen->getRequiresAuthentication() || $site->getPrivate()) {
            $username = $_SERVER['PHP_AUTH_USER'];
            $password = $_SERVER['PHP_AUTH_PW'];
            $user = null;
            if ($username !== null && $password !== null) {
                $user = (new SecurityManager())->getUserByName($username);
                if ($user) {
                    $upass = $user->password;
                    $upass = substr($upass, 0, 15);
                    if ($upass !== $password) {
                        $user = null;
                    }
                }
            }

            if ($site->getPrivate()) {
                if ($user && $user->id != 1) {
                    // check if member
                    $c = new Criteria();
                    $c->add("site_id", $site->getSiteId());
                    $c->add("user_id", $user->id);
                    $mem = MemberPeer::instance()->selectOne($c);
                    if (!$mem) {
                        // check if a viewer
                        $c = new Criteria();
                        $c->add("site_id", $site->getSiteId());
                        $c->add("user_id", $user->id);
                        $vi = SiteViewerPeer::instance()->selectOne($c);
                        if (!$vi) {
                            $user = null;
                        }
                    }
                }
            }

            if ($user == null) {
                header('WWW-Authenticate: Basic realm="Private"');
                header('HTTP/1.0 401 Unauthorized');
                header('Content-type: text/plain; charset=utf-8');
                echo _("This is a private feed. User authentication required via Basic HTTP Authentication. You cannot access it. Please go to 'Account settings' -> 'Notifications' to get the password if you believe you should be allowed.");
                exit();
            }
            $runData->setTemp("user", $user);
        }

        $logger->debug("OZONE initialized");

        $logger->info("Ozone engines successfully initialized");

        $rendered = $screen->render($runData);

        echo str_replace("%%%CURRENT_TIMESTAMP%%%", time(), $rendered);

        return $rendered;
    }
}
