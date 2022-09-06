<?php

namespace Wikidot\Utils;

use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Log;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\ModuleProcessor;
use Ozone\Framework\Ozone;
use Ozone\Framework\RunData;
use Ozone\Framework\WebFlowController;
use Wikidot\DB\SitePeer;
use Wikijump\Helpers\LegacyTools;

class WikiFlowController extends WebFlowController
{

    public function process()
    {
        global $timeStart;

        // quick fix to prevent recursive RSS access by Wikijump itself.
        if (strpos($_SERVER['HTTP_USER_AGENT'], 'MagpieRSS') !== false) {
            exit();
        }

        // initialize logging service
        Ozone::init();

        $runData = new RunData();
        $runData->init();
        Ozone::setRunData($runData);

        // Set cross-origin headers for improved security
        // https://scuttle.atlassian.net/browse/WJ-452
        $this->setCrossOriginHeaders();

        // check if site (Wiki) exists!
        $siteHost = $_SERVER["HTTP_HOST"];

        if (preg_match("/^([a-zA-Z0-9\-]+)\." . GlobalProperties::$URL_DOMAIN_PREG . "$/", $siteHost, $matches)==1) {
            $siteSlug = $matches[1];

            // select site based on the unix name

            // check memcached first!

            $mcKey = 'site..'.$siteSlug;
            $site = Cache::get($mcKey);

            if (!$site) {
                $c = new Criteria();
                $c->add("slug", $siteSlug);
                $c->add("site.deleted", false);
                $site = SitePeer::instance()->selectOne($c);
                if ($site) {
                    Cache::put($mcKey, $site, 864000);
                }
            }
        } else {
            // select site based on the custom domain
            $mcKey = 'site_cd..'.$siteHost;
            $site = Cache::get($mcKey);

            if (!$site) {
                $c = new Criteria();
                $c->add("custom_domain", $siteHost);
                $c->add("site.deleted", false);
                $site = SitePeer::instance()->selectOne($c);
                if ($site) {
                    Cache::put($mcKey, $site, 3600);
                }
            }

            if (!$site) {
                // check for redirects
                $c = new Criteria();
                $q = "SELECT site.* FROM site, domain_redirect WHERE domain_redirect.url='".db_escape_string($siteHost)."' " .
                        "AND site.deleted = false AND site.site_id = domain_redirect.site_id LIMIT 1";
                $c->setExplicitQuery($q);
                $site = SitePeer::instance()->selectOne($c);
                if ($site) {
                    $newUrl = GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().$_SERVER['REQUEST_URI'];
                    header("HTTP/1.1 301 Moved Permanently");
                    header("Location: ".$newUrl);
                    exit();
                }
            }

            GlobalProperties::$SESSION_COOKIE_DOMAIN = '.'.$siteHost;
        }

        if (!$site) {
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
        if ($_SERVER['HTTPS']) {
            if (!$sslMode) {
                // not enabled, redirect to http:
                header("HTTP/1.1 301 Moved Permanently");
                header("Location: ".GlobalProperties::$HTTP_SCHEMA . "://" . $_SERVER["HTTP_HOST"].$_SERVER['REQUEST_URI']);
                exit();
            } elseif ($sslMode == "ssl_only_paranoid") {
                // use secure authentication cookie
                // i.e. change authentication scheme
                GlobalProperties::$SESSION_COOKIE_NAME = GlobalProperties::$SESSION_COOKIE_NAME_SSL;
                GlobalProperties::$SESSION_COOKIE_SECURE = true;
            }
        } else {
            // page accessed via http (nonsecure)
            switch ($sslMode) {
                case 'ssl':
                    //enabled, but nonsecure allowed too.
                    break;
                case 'ssl_only_paranoid':
                case 'ssl_only':
                    header("HTTP/1.1 301 Moved Permanently");
                    header("Location: ".'https://'.$_SERVER["HTTP_HOST"].$_SERVER['REQUEST_URI']);
                    exit();
            }
        }

        // handle session at the begging of procession
        $runData->handleSessionStart();

        $template = $runData->getScreenTemplate();
        $classFile = $runData->getScreenClassPath();
        $class = LegacyTools::getNamespacedClassFromPath($classFile);
        Log::debug('[OZONE] Processing template', ['template' => $runData->getModuleTemplate(), 'class' => $class]);

        require_once($classFile);
        $screen = new $class();

        $rendered = $screen->render($runData);

        if ($rendered != null) {
            $runData->setTemp("jsInclude", array());
            // process modules...
            $moduleProcessor = new ModuleProcessor($runData);
            //$moduleProcessor->setJavascriptInline(true); // embed associated javascript files in <script> tags
            $moduleProcessor->setCssInline(true);
            $rendered = $moduleProcessor->process($rendered);

            $jss = $runData->getTemp("jsInclude");

            $jss = array_unique($jss);
            $incl = '';
            foreach ($jss as $js) {
                $incl .= '<script type="text/javascript" src="'.$js.'"></script>';
            }
            $rendered = preg_replace('/<\/head>/', $incl.'</head>', $rendered);
        }

        $runData->handleSessionEnd();

        // one more thing - some url will need to be rewritten if using HTTPS
        if ($_SERVER['HTTPS']) {
            // ?
            // scripts
            $rendered = preg_replace(';<script(.*?)src="'.GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST_PREG . '(.*?)</script>;s', '<script\\1src="https://' . GlobalProperties::$URL_HOST . '\\2</script>', $rendered);
            $rendered = preg_replace(';<link(.*?)href="'.GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST_PREG . '(.*?)/>;s', '<link\\1href="https://' . GlobalProperties::$URL_HOST . '\\2/>', $rendered);
            $rendered = preg_replace(';(<img\s+.*?src=")http(://' . GlobalProperties::$URL_HOST_PREG . '(.*?)/>);s', '\\1https\\2', $rendered);
            do {
                $renderedOld = $rendered;
                $rendered = preg_replace(';(<style\s+[^>]*>.*?@import url\()http(://' . GlobalProperties::$URL_HOST_PREG . '.*?</style>);si', '\\1https\\2', $rendered);
            } while ($renderedOld != $rendered);
        }

        echo str_replace("%%%CURRENT_TIMESTAMP%%%", time(), $rendered);

        return $rendered;
    }

    private function _fixHttpsStyles($matches)
    {
    }
}
