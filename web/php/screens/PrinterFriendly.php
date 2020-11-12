<?php
use DB\MemberPeer;
use DB\SiteViewerPeer;
use DB\PagePeer;

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
                if ($user && !$user->getSuperAdmin() && !$user->getSuperModerator()) {
                    // check if member
                    $c = new Criteria();
                    $c->add("site_id", $site->getSiteId());
                    $c->add("user_id", $user->getUserId());
                    $mem = MemberPeer::instance()->selectOne($c);
                    if (!$mem) {
                        // check if a viewer
                        $c = new Criteria();
                        $c->add("site_id", $site->getSiteId());
                        $c->add("user_id", $user->getUserId());
                        $vi = SiteViewerPeer::instance()->selectOne($c);
                        if (!$vi) {
                            $user = null;
                        }
                    }
                }
                if ($user == null) {
                    throw new ProcessException("This is a private wiki. Access is limited to selected users.");
                }
            }

            $wikiPage = WDStringUtils::toUnixName($wikiPage);
            $runData->setTemp("pageUnixName", $wikiPage);
            if ($wikiPage==="") {
                $wikiPage=$site->getDefaultPage();
            }

            $runData->contextAdd("wikiPageName", $wikiPage);
            // get wiki page from the database

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

            $licenseText = $category->getLicenseText();
            $runData->contextAdd("licenseText", $licenseText);

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
