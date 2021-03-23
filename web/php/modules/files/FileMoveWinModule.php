<?php

namespace Wikidot\Modules\Files;


use Wikidot\DB\FilePeer;
use Wikidot\DB\PagePeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionManager;

class FileMoveWinModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $fileId = $pl->getParameterValue("file_id");

        $file = FilePeer::instance()->selectByPrimaryKey($fileId);

        if ($file == null || $file->getSiteId() != $runData->getTemp("site")->getSiteId()) {
            throw new ProcessException(_("Error getting file information."), "no_file");
        }
        $page = PagePeer::instance()->selectByPrimaryKey($file->getPageId());
        if ($page == null || $page->getSiteId() != $runData->getTemp("site")->getSiteId()) {
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
