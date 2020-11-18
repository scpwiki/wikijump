<?php
use DB\PagePeer;
use DB\PageTagPeer;

class PageTagsModule extends SmartyModule
{

    public function build($runData)
    {
        $user = $runData->getUser();
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("pageId");

        $site = $runData->getTemp("site");

        if (!$pageId || !is_numeric($pageId)) {
            throw new ProcessException(_("The page cannot be found or does not exist."), "no_page");
        }

        $page = PagePeer::instance()->selectByPrimaryKey($pageId);

        if ($page == null || $page->getSiteId() != $site->getSiteId()) {
            throw new ProcessException(_("Error getting page information."), "no_page");
        }

        $category = $page->getCategory();

        WDPermissionManager::instance()->hasPagePermission('edit', $user, $category, $page);

        // get the tags now

        $c = new Criteria();
        $c->add("page_id", $pageId);

        $c->addOrderAscending("tag");

        $tags = PageTagPeer::instance()->select($c);

        $t2 = array();

        foreach ($tags as $t) {
            $t2[] = $t->getTag();
        }

        $t3 = implode(' ', $t2);

        $runData->contextAdd("tags", $t3);
    }
}
