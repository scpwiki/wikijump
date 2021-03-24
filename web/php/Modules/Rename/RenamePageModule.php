<?php

namespace Wikidot\Modules\Rename;

use Wikidot\DB\PagePeer;
use Exception;
use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionManager;

class RenamePageModule extends SmartyModule
{

    public function build($runData)
    {
        // only check for permissions
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("pageId");
        $page = PagePeer::instance()->selectByPrimaryKey($pageId);
        if ($page == null || $page->getSiteId() != $runData->getTemp("site")->getSiteId()) {
            throw new ProcessException(_("Error getting page information."), "no_page");
        }

        $delete = $pl->getParameterValue("delete");

        $user = $runData->getUser();

        if ($delete) {
            $newName = 'deleted:'.$page->getUnixName();
            $runData->contextAdd("delete", true);
        } else {
            $newName = $page->getUnixName();
        }

        $category = $page->getCategory();
        $runData->contextAdd("page", $page);

        $runData->contextAdd("newName", $newName);

        // now check for permissions!!!

        WDPermissionManager::instance()->hasPagePermission('move', $user, $category, $page);

        $canDelete = true;
        try {
            WDPermissionManager::instance()->hasPagePermission('delete', $user, $category, $page);
        } catch (Exception $e) {
            $canDelete = false;
        }

        $runData->contextAdd("canDelete", $canDelete);

        // check if belongs to a special category...
        $categoryName = $category->getName();
        if ($categoryName == "forum") {
            $runData->contextAdd("isForum", true);
        }
        if ($categoryName == "admin") {
            $runData->contextAdd("isAdmin", true);
        }
    }
}
