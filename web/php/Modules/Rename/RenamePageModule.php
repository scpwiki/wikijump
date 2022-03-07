<?php

namespace Wikidot\Modules\Rename;

use Exception;
use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionManager;
use Wikijump\Services\Deepwell\Models\Page;

class RenamePageModule extends SmartyModule
{

    public function build($runData)
    {
        // only check for permissions
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("pageId");
        $page = Page::findIdOnly($pageId);
        if ($page === null || $page->getSiteId() !== $runData->getTemp("site")->getSiteId()) {
            throw new ProcessException(_("Error getting page information."), "no_page");
        }

        $delete = $pl->getParameterValue("delete");

        $user = $runData->getUser();

        if ($delete) {
            $newName = 'deleted:'.$page->slug;
            $runData->contextAdd("delete", true);
        } else {
            $newName = $page->slug;
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
