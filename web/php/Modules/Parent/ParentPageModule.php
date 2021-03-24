<?php

namespace Wikidot\Modules\Parent;

use Wikidot\DB\PagePeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionManager;

class ParentPageModule extends SmartyModule
{

    public function build($runData)
    {
        $pageId = $runData->getParameterList()->getParameterValue("page_id");

        $page = PagePeer::instance()->selectByPrimaryKey($pageId);
        if ($page == null || $page->getSiteId() != $runData->getTemp("site")->getSiteId()) {
            throw new ProcessException(_("Error getting page information."), "no_page");
        }

        $user = $runData->getUser();
        // check permissions now
        $category = $page->getCategory();
        // now check for permissions!!!
        WDPermissionManager::instance()->hasPagePermission('edit', $user, $category, $page);

        if ($page->getParentPageId() !== null) {
            $parentPage = PagePeer::instance()->selectByPrimaryKey($page->getParentPageId());
            $runData->contextAdd("parentPageName", $parentPage->getUnixName());
        }
    }
}
