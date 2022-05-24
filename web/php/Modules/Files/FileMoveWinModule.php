<?php

namespace Wikidot\Modules\Files;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionManager;
use Wikijump\Services\Deepwell\Models\File;
use Wikijump\Services\Deepwell\Models\Page;

class FileMoveWinModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $fileId = $pl->getParameterValue("file_id");
        $site = $runData->getTemp("site");

        $file = File::findId($fileId);
        if ($file === null || $file->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("Error getting file information."), "no_file");
        }

        $page = Page::findIdOnly($file->getPageId());
        if ($page === null || $page->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("Error getting page information."), "no_page");
        }

        // check permissions
        $category = $page->getCategory();
        // now check for permissions!!!
        $user = $runData->getUser();
        WDPermissionManager::instance()->hasPagePermission('move_file', $user, $category);

        $runData->contextAdd("file", $file);
        $runData->contextAdd("page", $page);
    }
}
