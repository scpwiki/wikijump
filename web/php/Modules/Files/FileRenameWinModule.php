<?php

namespace Wikidot\Modules\Files;

use Ozone\Framework\SmartyModule;
use Wikidot\DB\FilePeer;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionManager;
use Wikijump\Services\Deepwell\Models\Page;

class FileRenameWinModule extends SmartyModule
{
    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $fileId = $pl->getParameterValue("file_id");

        $file = FilePeer::instance()->selectByPrimaryKey($fileId);
        if ($file == null || $file->getSiteId() != $runData->getTemp("site")->getSiteId()) {
            throw new ProcessException(_("Error getting file information."), "no_file");
        }
        $page = Page::findIdOnly($file->getPageId());
        if ($page == null || $page->getSiteId() != $runData->getTemp("site")->getSiteId()) {
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
