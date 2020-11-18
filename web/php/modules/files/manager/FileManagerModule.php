<?php
class FileManagerModule extends SmartyModule
{

    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $settings = $site->getSettings();

        $runData->contextAdd("site", $site);
        $runData->contextAdd("settings", $settings);

        $totalSize = FileHelper::totalSiteFilesSize($site->getSiteId());
        $allowed = $settings->getFileStorageSize();

        $maxUpload = min($allowed - $totalSize, 5242880);

        $numberOfFiles = FileHelper::totalSiteFileNumber($site->getSiteId());

        $runData->contextAdd("totalSiteSize", FileHelper::formatSize($totalSize));
        $runData->contextAdd("numberOfFiles", $numberOfFiles);
        $runData->contextAdd("totalSiteAllowedSize", FileHelper::formatSize($allowed));
        $runData->contextAdd("availableSiteSize", FileHelper::formatSize($allowed - $totalSize));

        $runData->contextAdd("maxUpload", $maxUpload);
        $runData->contextAdd("maxUploadString", FileHelper::formatSize($maxUpload));
    }
}
