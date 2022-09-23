<?php

namespace Wikidot\Modules\Files;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionManager;
use Wikijump\Services\Deepwell\Models\File;
use Wikijump\Services\Deepwell\Models\Page;

class FileRenameWinModule extends SmartyModule
{
    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");
        $fileId = $pl->getParameterValue("file_id");

        $file = File::findId($fileId);
        if ($file === null || $file->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("Error getting file information."), "no_file");
        }

        $page = Page::findIdOnly($file->getPageId());
        if ($page === null || $page->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("Error getting file information."), "no_page");
        }
        // check permissions
        $category = $page->getCategory();
        // now check for permissions!!!
        $user = $runData->getUser();
        WDPermissionManager::instance()->hasPagePermission('rename_file', $user, $category);

        $runData->contextAdd("file", $file);
    }
}
