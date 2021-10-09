<?php

namespace Wikidot\Screens;

use Exception;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Ozone\Framework\PathManager;
use Ozone\Framework\Screen;
use Wikidot\DB\MemberPeer;
use Wikidot\DB\SiteViewerPeer;
use Wikidot\DB\PagePeer;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDStringUtils;

class PrinterFriendly extends Screen
{

    public function render($runData)
    {

        try {
            // get site
            $site = $runData->getTemp("site");
            $runData->contextAdd("site", $site);

            $pl = $runData->getParameterList();

            $wikiPage = $pl->getParameterValue("wiki_page");

            if ($site->getPrivate()) {
                $user = $runData->getUser();
                if ($user->id != 1) {
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
                    throw new ProcessException("This is a private Wiki. Access is limited to selected users.");
                }
            }

            $wikiPage = WDStringUtils::toUnixName($wikiPage);
            $runData->setTemp("pageUnixName", $wikiPage);
            if ($wikiPage==="") {
                $wikiPage=$site->getDefaultPage();
            }

            $runData->contextAdd("wikiPageName", $wikiPage);
            // get Wiki page from the database

            $page = PagePeer::instance()->selectByName($site->getSiteId(), $wikiPage);

            if ($page == null) {
                throw new ProcessException("No such page");
            } else {
                // page exists!!! wooo!!!

                $runData->setTemp("page", $page);
                $GLOBALS['page'] = $page;

                $compiled = $page->getCompiled();

                $runData->contextAdd("wikiPage", $page);
                $runData->contextAdd("screen_placeholder", $compiled->getText());

                $category = $page->getCategory();
                $runData->setTemp("category", $category);
            }

            $runData->contextAdd("category", $category);

            // GET THEME for the category

            $theme = $category->getTheme();
            $runData->contextAdd("theme", $theme);

            // GET LICENSE for the category

            $licenseHtml = $category->getLicenseHtml();
            $runData->contextAdd("licenseHtml", $licenseHtml);

            $smarty = Ozone::getSmarty();

            // put context into context

            $context = $runData->getContext();
            if ($context !== null) {
                foreach ($context as $key => $value) {
                    $smarty->assign($key, $value);
                }
            }

            $layoutFile = PathManager::layoutTemplate("PrintLayout");
            $out = $smarty->fetch($layoutFile);

            return $out;
        } catch (Exception $e) {
            $out = $e->getMessage();
            return $out;
        }
    }
}
