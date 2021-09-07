<?php
declare(strict_types=1);

namespace Wikijump\Helpers;

use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Log;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\ModuleProcessor;
use Ozone\Framework\Ozone;
use Ozone\Framework\RunData;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\ForumThreadPeer;
use Wikidot\DB\MemberPeer;
use Wikidot\DB\Page;
use Wikidot\DB\PageCompiled;
use Wikidot\DB\PagePeer;
use Wikidot\DB\PageTagPeer;
use Wikidot\DB\Site;
use Wikidot\DB\SitePeer;
use Wikidot\DB\SiteViewerPeer;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\UploadedFileFlowController;
use Wikidot\Utils\WDPermissionManager;
use Wikidot\Utils\WDStringUtils;
use Wikijump\Models\User;

/** A collection of static methods to smooth the transition to Wikijump code. */
final class LegacyTools
{
    // Disallow creating instances
    private function __construct() {}

    /**
     * A function to take an absolute path to a file and transform it to a properly namespaced class.
     *
     * This may need to be adjusted occasionally if namespaces change (check composer.json) or things move around.
     *
     * @param string $path An absolute path e.g., /var/www/path/to/modules/file.php
     * @return string A namespaced legacy class e.g., Wikidot\Modules\File
     */
    public static function getNamespacedClassFromPath(string $path) : string
    {
        $offset = strlen(dirname(__FILE__, 3)); // Get the length of the string of the absolute path 3 levels up from this file.
        $unique_path = substr($path, $offset, -4); // Chop off that length and the last 4 characters. (.php)
        $unique_path = str_replace('/', '\\', $unique_path);
        $translations = [
            "\\php\\" => "Wikidot\\",
            "\\lib\\ozoneframework\\php\\core\\" => "Ozone\\Framework\\",
            "\\lib\\ozoneframework\\php\\Template\\" => "Ozone\\Framework\\Template\\"
        ];

        return strtr($unique_path, $translations);
    }

    /**
     * Determine whether an account is one of the system-generated accounts.
     * @param int $id
     * @return bool
     */
    public static function isSystemAccount(int $id) : bool
    {
        return ($id === User::ANONYMOUS_USER || $id === User::AUTOMATIC_USER);
    }

    /**
     * Bootstrap a runData instance and generate the needed vars to give to a blade template.
     * @return array|string
     * @throws \Wikidot\Utils\ProcessException
     */
    public static function generateScreenVars()
    {
        /**
         * Create a RunData instance.
         */
        $runData = new RunData();
        $runData->init();
        Ozone::setRunData($runData);
        Log::debug('runData object created and initialized in LegacyTools::generateScreenVars()');

        /**
         * Determine if the host we received the connection on has a site associated with it.
         */
        $siteHost = $_SERVER["HTTP_HOST"];
        if (preg_match("/^([a-zA-Z0-9\-]+)\." . GlobalProperties::$URL_DOMAIN_PREG . "$/", $siteHost, $matches)==1) {
            $siteUnixName = $matches[1];
        }

        $c = new Criteria();
        $c->add("unix_name", $siteUnixName);
        $c->add("site.deleted", false);
        $site = SitePeer::instance()->selectOne($c);
        if ($site === null) {
            $c = new Criteria();
            $c->add("custom_domain", $siteHost);
            $c->add("site.deleted", false);

            /** @var Site? $site */
            $site = SitePeer::instance()->selectOne($c);
        }
        if ($site === null) {
            // check for redirects
            $c = new Criteria();
            $q = "SELECT site.* FROM site, domain_redirect WHERE domain_redirect.url='".db_escape_string($siteHost)."' " .
                "AND site.deleted = false AND site.site_id = domain_redirect.site_id LIMIT 1";
            $c->setExplicitQuery($q);
            /** @var Site? $site */
            $site = SitePeer::instance()->selectOne($c);
            if ($site !== null) {
                $newUrl = GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().$_SERVER['REQUEST_URI'];
                header("HTTP/1.1 301 Moved Permanently");
                header("Location: ".$newUrl);
                exit();
            }
        }

        if ($site === null) {
            // echo file_get_contents(WIKIJUMP_ROOT."/resources/views/site_not_exists.html");
            return null;
        }

        /**
         * Set site params
         */
        $runData->setTemp("site", $site);
        $GLOBALS['siteId'] = $site->getSiteId();
        $GLOBALS['site'] = $site;

        /**
         * Set language params
         */
        $lang = $site->getLanguage();
        $runData->setLanguage($lang);
        $GLOBALS['lang'] = $lang;
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
        $gdomain = 'messages';
        bindtextdomain($gdomain, WIKIJUMP_ROOT.'/locale');
        textdomain($gdomain);

        /**
         * Get session from DB
         */
        $runData->handleSessionStart();

        /**
         * Begin rendering WikiLayout (views/layouts/legacy.blade.php) vars
         */
        $return = [];
        $site = $runData->getTemp("site");
        $runData->contextAdd("site", $site);
        $return['site'] = $site;
        /** Normally we would handle notifications here in the legacy flow. */
        $pl = $runData->getParameterList();
        $wikiPage = $pl->getParameterValue("wiki_page");

        /**
         * Determine page visibility
         */
        $privateAccessGranted = true;
        // check if the site is private
        if ($site->getPrivate()) {
            $user = $runData->getUser();
            if ($user->id !== 1) {
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
            if ($user == null) {
                $wikiPage = $site->getSettings()->getPrivateLandingPage();
                $privateAccessGranted = false;
            }
        }

        // Gets parameters (e.g. /noredirect/true), if any
        $pageParameters = self::getPageParameters();
        $return['pageParameters'] = $pageParameters;

        /**
         * Get Page
         */
        $wikiPage = self::redirectToNormalUrl($site, $wikiPage, $pageParameters);
        $runData->setTemp("pageUnixName", $wikiPage);
        $runData->contextAdd("wikiPageName", $wikiPage);
        $return['wikiPageName'] = $wikiPage;
        $settings = $site->getSettings();
        /** @var ?Page $page */
        $page = PagePeer::instance()->selectByName($site->getSiteId(), $wikiPage);
        if ($page == null) {
            $runData->contextAdd("pageNotExists", true);
            $return['pageNotExists'] = true;
            // get category based on suggested page name

            if (strpos($wikiPage, ":") != false) {
                $tmp0 = explode(':', $return['wikiPage']);
                $categoryName = $tmp0[0];
            } else {
                $categoryName = "_default";
            }
            $category = CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId());
            if ($category == null) {
                $category = CategoryPeer::instance()->selectByName('_default', $site->getSiteId());
            }
            $runData->setTemp("category", $category);
        } else {
            // page exists!!! wooo!!!

            $runData->setTemp("page", $page);
            $GLOBALS['page'] = $page;

            /** @var PageCompiled $compiled */
            $compiled = $page->getCompiled();

            $runData->contextAdd("wikiPage", $page);
            $return['wikiPage'] = $page;
            $runData->contextAdd("pageContent", $compiled->getText());
            $return['pageContent'] = $compiled->getText();

            $category = $page->getCategory();
            $runData->setTemp("category", $category);

            // show options?
            $showPageOptions = true;
            $runData->contextAdd("showPageoptions", $showPageOptions);
            $return['showPageoptions'] = true;

            // get the tags
            $c = new Criteria();
            $c->add("page_id", $page->getPageId());
            $c->addOrderAscending("tag");
            $tags = PageTagPeer::instance()->select($c);
            $t2 = array();
            foreach ($tags as $t) {
                $t2[] = $t->getTag();
            }
            $runData->contextAdd("tags", $t2);
            $return['tags'] = $tags;

            // has discussion?
            if ($page->getThreadId()!== null) {
                $thread = ForumThreadPeer::instance()->selectByPrimaryKey($page->getThreadId());
                if ($thread == null) {
                    $page->setThreadId(null);
                    $page->save();
                } else {
                    $page->setTemp("numberPosts", $thread->getNumberPosts());
                }
            }

            // look for parent pages (and prepare breadcrumbs)
            if ($page->getParentPageId()) {
                $breadcrumbs = array();
                $ppage = PagePeer::instance()->selectByPrimaryKey($page->getParentPageId());
                array_unshift($breadcrumbs, $ppage);
                $bcount = 0;
                while ($ppage->getParentPageId() && $bcount<=4) {
                    $ppage = PagePeer::instance()->selectByPrimaryKey($ppage->getParentPageId());
                    array_unshift($breadcrumbs, $ppage);
                    $bcount++;
                }
                $runData->contextAdd("breadcrumbs", $breadcrumbs);
                $return['breadcrumbs'] = $breadcrumbs;
            }
        }

        $runData->contextAdd("category", $category);
        $return['category'] = $category;

        // GET THEME for the category

        $theme = $category->getTheme();
        $runData->contextAdd("theme", $theme);
        $return['theme'] = $theme;

        // GET LICENSE for the category

        $licenseText = $category->getLicenseText();
        $runData->contextAdd("licenseText", $licenseText);
        $return['licenseText'] = $licenseText;

        // show nav elements?

        if ($privateAccessGranted || !$settings->getHideNavigationUnauthorized()) {
            if ($theme->getUseSideBar()) {
                $sideBar1 = $category->getSidePage();
                if ($sideBar1 !== null) {
                    $sideBar1Compiled = $sideBar1->getCompiled();
                    $ccc =  $sideBar1Compiled->getText();
                    $ccc = preg_replace('/id="[^"]*"/', '', $ccc);
                    $runData->contextAdd("sideBar1Content", $ccc);
                    $return['sideBar1Content'] = $ccc;
                }
            }
            if ($theme->getUseTopBar()) {
                $topBar = $category->getTopPage();
                if ($topBar !== null) {
                    $topBarCompiled = $topBar->getCompiled();
                    $ccc =  $topBarCompiled->getText();
                    $ccc = preg_replace('/id="[^"]*"/', '', $ccc);
                    $runData->contextAdd("topBarContent", $ccc);
                    $return['topBarContent'] = $ccc;
                }
            }
        }

        /**
         * Process Modules
         */
        $runData->setTemp("jsInclude", array());
        // process modules...
        $moduleProcessor = new ModuleProcessor($runData);
        //$moduleProcessor->setJavascriptInline(true); // embed associated javascript files in <script> tags
        $moduleProcessor->setCssInline(true);
        $return['sideBar1Content'] = $moduleProcessor->process($return['sideBar1Content']);
        $return['topBarContent'] = $moduleProcessor->process($return['topBarContent']);
        $return['pageContent'] = $moduleProcessor->process($return['pageContent']);

        $jss = $runData->getTemp("jsInclude");

        $jss = array_unique($jss);
        $incl = '';
        foreach ($jss as $js) {
            $incl .= '<script type="text/javascript" src="'.$js.'"></script>';
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

        echo str_replace("%%%CURRENT_TIMESTAMP%%%", (string)time(), $rendered);

//        dd($rendered);

        /**
         * Custom Domain Script module injection
         */
        if (!$runData->getUser() && preg_match('/^([a-zA-Z0-9\-]+)\.' . GlobalProperties::$URL_DOMAIN .'$/', $_SERVER["HTTP_HOST"], $matches) !==1) {
            $runData->contextAdd("useCustomDomainScript", true);
            $return['useCustomDomainScript'] = true;
            $runData->contextAdd("useCustomDomainScriptSecure", $_SERVER['HTTPS']);
            $return['useCustomDomainScriptSecure'] = isset($_SERVER['HTTPS']);
            $runData->contextAdd("site", $runData->getTemp("site"));
        }

        /**
         * File Auth Script module injection
         */
        $u = new UploadedFileFlowController();
        if ($runData->getUser() && $site->getPrivate() && $u->userAllowed($runData->getUser(), $site)) {
            $pwdomain = $site->getUnixName() . "." . GlobalProperties::$URL_UPLOAD_DOMAIN;
            $pwproto = ($_SERVER["HTTPS"]) ? "https" : "http";
            $pwurl = "$pwproto://$pwdomain/filesauth.php";

            $runData->contextAdd("usePrivateWikiScript", true);
            $return['usePrivateWikiScript'] = true;
            $runData->contextAdd("privateWikiScriptUrl", $pwurl);
            $return['privateWikiScriptUrl'] = $pwurl;
        }

        /**
         * Login Status module injection
         */

        $return['login'] = view('legacy.loginstatus');

        /**
         * Page Options Bottom module injection
         */
        $pl = $runData->getParameterList();
        $pageName = $runData->getTemp("pageUnixName");

        $page = $runData->getTemp("page");//$pl->getParameterValue("page", "MODULE");

        // get category name and get the category by name.
        // this should be enchanced to use memcache later
        // to get category to avoid db connection.

        // extract category name
        if (strpos($pageName, ':') != false) {
            // ok, there is category!
            $exp = explode(':', $pageName);
            $categoryName = $exp[0];
        } else {
            $categoryName = "_default";
        }
        $site = $runData->getTemp("site");
        $category = CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId());
        $user = $runData->getUser();

        $pm = new WDPermissionManager();
        $pm->setThrowExceptions(false);
        $pm->setCheckIpBlocks(false); // to avoid database connection.
        if (!$pm->hasPagePermission('options', $user, $category, $pageName, $site)) {
            return '';
        }

        $showDiscuss = $category->getShowDiscuss();
        if ($showDiscuss) {
            $threadId = $wikiPage->getThreadId();
            $pageUnixName = $wikiPage->getUnixName();
        }

        $showRate = $category->getRatingEnabledEff();

        // now a nasty part - make it inline such that
        // the Smarty engine does need to be initialized.
        // nasty way but saves a LOT of time with caching enabled.

        $otext = '';

        if ($page) {
            $otext .=   '<div id="page-info">'.
                _('page_revision').': '.$page->getRevisionNumber().', '.
                _('last_edited').': <span class="odate">'.
                $page->getDateLastEdited()->getTimestamp().
                '|%e %b %Y, %H:%M %Z (%O '._('ago').')</span>'.
                '</div>';
        }

        $otext .= '
<div id="page-options-bottom"  class="page-options-bottom">
	<a href="javascript:;" id="edit-button">'._('edit').'</a>';

        if ($showRate&&$page) {
            $otext .=   '<a href="javascript:;" id="pagerate-button">'._('rate').' (<span id="prw54355">'.($page->getRate() > 0 && $category->getRatingType() != "S" ?'+':''). ($category->getRatingType() == "S" ? $page->getRate() : round($page->getRate())) .'</span>)</a>';
        }

        $otext .= '<a href="javascript:;" id="tags-button">'._('tags').'</a>';

        if ($showDiscuss&&$page) {
            if ($threadId) {
                $no = $page->getTemp("numberPosts");
                $otext.='<a href="/forum/t-'.$threadId.'/'.$pageUnixName.'"  id="discuss-button">'._('discuss').' ('.$no.')</a>';
            } else {
                $otext.='<a href="javascript:;" id="discuss-button" onclick="Wikijump.page.listeners.createPageDiscussion(event)">'._('discuss').'</a> ';
            }
        }

        $otext .= '
	<a href="javascript:;" id="history-button">'._('history').'</a>
	<a href="javascript:;" id="files-button">'._('files').'</a> ' .
            '<a href="javascript:;" id="print-button">'._('print').'</a> ' .
            '<a href="javascript:;" id="site-tools-button">'._('site tools').'</a>';
        $otext .= '<a href="javascript:;" id="more-options-button">+&nbsp;'._('options').'</a>
</div>
<div id="page-options-bottom-2" class="page-options-bottom" style="display:none">
	<a href="javascript:;" id="edit-sections-button">'._('edit sections').'</a>
	<a href="javascript:;" id="edit-append-button">'._('append').'</a>
	<a href="javascript:;" id="backlinks-button">'._('backlinks').'</a>
	<a href="javascript:;" id="view-source-button">'._('view source').'</a>
	<a href="javascript:;" id="parent-page-button">'._('parent').'</a>
	<a href="javascript:;" id="page-block-button">'._('block').'</a>
	<a href="javascript:;" id="rename-move-button">'._('rename').'</a>
	<a href="javascript:;" id="delete-button">'._('delete').'</a>
</div>
<div id="page-options-area-bottom">
</div>
';

        $return['pageOptions'] = $otext;

        return $return;
    }

    /**
     * Following the page name, URLs may include "/key/value" type
     * parameters for additional values, for instance "/noredirect/true".
     *
     * This method extracts them, if there are any.
     *
     * @return string The page parameters part of the URI (if any)
     */
    public static function getPageParameters(): string
    {
        return preg_replace('/^\/[^\/]+/u', '', $_SERVER['REQUEST_URI']);
    }

    /**
     * This method gets the normalized page slug, and if it differs
     * from what is entered in the URL, redirects the user to that address.
     *
     * @return string The normalized page name
     */
    public static function redirectToNormalUrl(Site $site, string $slug, string $pageParameters): string
    {
        if ($slug === '') {
            $slug = $site->getDefaultPage();
        }

        $slugNormal = WDStringUtils::toUnixName($slug);
        if ($slug !== $slugNormal) {
            // Redirect to the normalized version
            $newUrl = GlobalProperties::$HTTP_SCHEMA . '://' . $site->getDomain() . $wikiPageNormal . $pageParameters;
            header('HTTP/1.1 301 Moved Permanently');
            header('Location: ' . $newUrl);
            exit();
        }

        return $slugNormal;
    }
}
