<?php

namespace Wikidot\Modules\Files;


use Wikidot\DB\PagePeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\FileHelper;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionManager;

class FileUploadModule extends SmartyModule
{

    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("pageId");
        $page = PagePeer::instance()->selectByPrimaryKey($pageId);
        if ($page == null || $page->getSiteId() != $site->getSiteId()) {
            throw new ProcessException(_("Problem selecting destination page."), "no_page");
        }

        $category = $page->getCategory();
        // now check for permissions!!!
        $user = $runData->getUser();
        WDPermissionManager::instance()->hasPagePermission('attach_file', $user, $category, $page);

        $totalSize = FileHelper::totalSiteFilesSize($site->getSiteId());
        $allowed = $site->getSettings()->getFileStorageSize();

        $maxUpload = min($allowed - $totalSize, $site->getSettings()->getMaxUploadFileSize());

        $runData->contextAdd("totalSiteSize", FileHelper::formatSize($totalSize));
        $runData->contextAdd("totalSiteAllowedSize", FileHelper::formatSize($allowed));
        $runData->contextAdd("availableSiteSize", FileHelper::formatSize($allowed - $totalSize));

        $runData->contextAdd("maxUpload", $maxUpload);
        $runData->contextAdd("maxUploadString", FileHelper::formatSize($maxUpload));
    }
}
