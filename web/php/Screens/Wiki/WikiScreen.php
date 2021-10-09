<?php

namespace Wikidot\Screens\Wiki;

use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\DB;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\Ozone;
use Ozone\Framework\PathManager;
use Ozone\Framework\Screen;
use Wikidot\DB\MemberPeer;
use Wikidot\DB\SiteViewerPeer;
use Wikidot\DB\PagePeer;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\ForumThreadPeer;
use Wikidot\DB\NotificationPeer;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\WDStringUtils;
use Wikijump\Helpers\LegacyTools;
use Wikijump\Models\UserMessage;

class WikiScreen extends Screen
{

    private $vars = array();

    public function render($runData)
    {
        // get site
        $site = $runData->getTemp("site");
        $runData->contextAdd("site", $site);

        $this->handleNotifications($runData);

        $pl = $runData->getParameterList();

        $wikiPage = $pl->getParameterValue("wiki_page");

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

        $pageParameters = LegacyTools::getPageParameters();
        $wikiPage = LegacyTools::redirectToNormalUrl($site, $wikiPage, $pageParameters);
        $runData->setTemp("pageUnixName", $wikiPage);

        if ($runData->getAction() == null
                && $runData->getRequestMethod() == "GET"
                && $privateAccessGranted
            ) {
            // try to get content from the memorycache server

            $mcKey = 'page..'.$site->getUnixName().'..'.$wikiPage;

            if (strpos($wikiPage, ":") != false) {
                $tmp0 = explode(':', $wikiPage);
                $categoryName = $tmp0[0];
            } else {
                $categoryName = "_default";
            }
            $aKey = 'category_lc..'.$site->getUnixName().'..'.$categoryName;
            $changeTime = Cache::get($aKey);
            $cachedPage = Cache::get($mcKey);
            if ($cachedPage !== false && $changeTime && $changeTime <= $cachedPage['timestamp']) {
                $runData->setTemp("page", $cachedPage['page']);
                $GLOBALS['page'] = $cachedPage['page'];

                $out = $cachedPage['content'];
                if ($this->vars['notificationsDialog']) {
                    $out = preg_replace(
                        '/
                        <div id="account-notifications-dummy" style="display:none"><\/div>
                        /x',
                        '<div id="notifications-dialog" style="display:none">'.
                        $this->vars['notificationsDialog'].'</div>',
                        $out,
                        1
                    );
                }
                return $out;
            } else {
                $storeLater = true;
            }
        }

        $runData->contextAdd("wikiPageName", $wikiPage);

        $settings = $site->getSettings();

        // get Wiki page from the database
        $page = PagePeer::instance()->selectByName($site->getSiteId(), $wikiPage);

        if ($page == null) {
            $runData->contextAdd("pageNotExists", true);
            // get category based on suggested page name

            if (strpos($wikiPage, ":") != false) {
                $tmp0 = explode(':', $wikiPage);
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

            $compiled = $page->getCompiled();

            $runData->contextAdd("wikiPage", $page);
            $runData->contextAdd("pageContent", $compiled->getText());

            $category = $page->getCategory();
            $runData->setTemp("category", $category);

            // show options?
            $showPageOptions = true;
            $runData->contextAdd("showPageoptions", $showPageOptions);

            // Get the tags and convert them to string.
            $tags = PagePeer::getTags($pageId);
            if(!empty($tags)) {
                $tags = implode(" ", $tags);
            }
            $runData->contextAdd("tags", $tags);

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
            }
        }

        $runData->contextAdd("category", $category);

        // GET THEME for the category

        $theme = $category->getTheme();
        $runData->contextAdd("theme", $theme);

        // GET LICENSE for the category

        $licenseHtml = $category->getLicenseHtml();
        $runData->contextAdd("licenseHtml", $licenseHtml);

        // show nav elements?

        if ($privateAccessGranted || !$settings->getHideNavigationUnauthorized()) {
            if ($theme->getUseSideBar()) {
                $sideBar1 = $category->getSidePage();
                if ($sideBar1 !== null) {
                    $sideBar1Compiled = $sideBar1->getCompiled();
                    $ccc =  $sideBar1Compiled->getText();
                    $ccc = preg_replace('/id="[^"]*"/', '', $ccc);
                    $runData->contextAdd("sideBar1Content", $ccc);
                }
            }
            if ($theme->getUseTopBar()) {
                $topBar = $category->getTopPage();
                if ($topBar !== null) {
                    $topBarCompiled = $topBar->getCompiled();
                    $ccc =  $topBarCompiled->getText();
                    $ccc = preg_replace('/id="[^"]*"/', '', $ccc);
                    $runData->contextAdd("topBarContent", $ccc);
                }
            }
        }

        // check wether to include a special JS file for custom domains or a special JS file for private files
        //if (preg_match('/^([a-zA-Z0-9\-]+)\.' . GlobalProperties::$URL_DOMAIN_PREG . '$/',$_SERVER["HTTP_HOST"], $matches) !==1) {
        //  $runData->contextAdd("useCustomDomainScript", true);
        //}

        $smarty = Ozone::getSmarty();

        // put context into context

        $context = $runData->getContext();
        if ($context !== null) {
            foreach ($context as $key => $value) {
                $smarty->assign($key, $value);
            }
        }

        $templateFile = PathManager::screenTemplate("Wiki/WikiScreen");
        $screenContent = $smarty->fetch($templateFile);

        $smarty->assign("screen_placeholder", $screenContent);
        $layoutFile = PathManager::layoutTemplate("WikiLayout");
        $out = $smarty->fetch($layoutFile);

        if ($storeLater) {
            if (!$changeTime) {
                Cache::put($aKey, time(), 864000);
            }
            Cache::put($mcKey, array("page" =>$page, "content" => $out, "timestamp" => time()), 864000);
        }

        if ($this->vars['notificationsDialog']) {
            $out = preg_replace(
                ';<div id="account-notifications-dummy" style="display:none"></div>;',
                '<div id="notifications-dialog" style="display:none">'.
                            $this->vars['notificationsDialog'].'</div>',
                $out,
                1
            );
        }

        return $out;
    }

    private function handleNotifications($runData)
    {
        // check not earlier than 2 minutes after the previous check
        $user = $runData->getUser();
        if ($user == null) {
            return;
        }

        // get last check date
        $lastCheck = $_COOKIE['lastncheck'];
        if ($lastCheck !== null && is_numeric($lastCheck) && time() - $lastCheck < 120) {
            return;
        }

        setsecurecookie('lastncheck', time(), time() + 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);
        // ok. go get the notifications now.

        $c = new Criteria();
        $c->add("user_id", $user->id);
        $c->add("notify_online", true);
        $c->addOrderDescending("notification_id");

        $nots = NotificationPeer::instance()->select($c);

        if (count($nots) == 0) {
            return;
        }

        if (count($nots)>0) {
            $q = "UPDATE notification SET notify_online=FALSE, notify_email=FALSE " .
                    "WHERE user_id='".$user->id."' AND " .
                    "notify_online = TRUE";
            $db = Database::connection();
            $db->query($q);
        }

        $nots2 = array();

        foreach ($nots as &$not) {
            if ($not->getType() == "new_private_message") {
                // check if the message is read or still new
                $extra = $not->getExtra();
                $pm = UserMessage::find($extra['message_id']);
                if ($pm && $pm->isUnread()) {
                    $body = $not->getBody();
                    $body = preg_replace('/<br\/>Preview.*$/sm', '', $body);
                    $body = preg_replace(';You have.*?<br/>;sm', '', $body);
                    $not->setBody($body);
                    $nots2[] = $not;
                }
            } else {
                $nots2[] = $not;
            }
        }

        if (count($nots2)==0) {
            return;
        }

        $lang = $user->language;

        switch ($lang) {
            case 'pl':
                $glang="pl_PL";
                $wp = "pl";
                break;
            case 'en':
                $glang="en_US";
                $wp = "www";
                break;
        }

        $runData->setLanguage($lang);
        putenv("LANG=$glang");
        putenv("LANGUAGE=$glang");
        setlocale(LC_ALL, $glang.'.UTF-8');

        // get Smarty and render a dialog
        $smarty = Ozone::getSmartyPlain();
        $dialogTemplateFile  = PathManager::screenTemplate("NotificationDialog");

        $count = count($nots2);
        if ($count>3) {
            $nots2  = array_slice($nots2, 0, 3);
            $smarty->assign("more", $count -3);
        }
        $smarty->assign("count", $count);

        $smarty->assign("notifications", $nots2);

        $out = $smarty->fetch($dialogTemplateFile);

        $this->vars['notificationsDialog'] = $out;

        $lang = $GLOBALS['lang'];

        switch ($lang) {
            case 'pl':
                $glang="pl_PL";
                break;
            case 'en':
                $glang="en_US";
                break;
        }

        $runData->setLanguage($lang);
        putenv("LANG=$glang");
        putenv("LANGUAGE=$glang");
        setlocale(LC_ALL, $glang.'.UTF-8');
    }
}
