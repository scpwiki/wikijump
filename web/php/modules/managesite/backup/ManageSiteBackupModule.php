<?php

namespace Wikidot\Modules\ManageSite\Backup;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\SiteBackupPeer;
use Wikidot\Utils\FileHelper;
use Wikidot\Utils\ManageSiteBaseModule;

class ManageSiteBackupModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        // get current backups

        $site = $runData->getTemp("site");

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());

        $sb = SiteBackupPeer::instance()->selectOne($c);

        if ($sb) {
            if ($sb->getStatus() == "completed") {
                // get backup file size
                $path = WIKIJUMP_ROOT.'/web/files--sites/'.$site->getUnixName().'/backup/'.$sb->getRand().'/backup.zip';
                // check if file exists
                if (!file_exists($path)) {
                    // in case something failed
                    SiteBackupPeer::instance()->delete($c);
                    $sb = null;
                } else {
                    $size = filesize($path);
                    $sizeFormatted = FileHelper::formatSize($size);

                    $runData->contextAdd('size', $sizeFormatted);
                }
            }
        }

        $runData->contextAdd("site", $site);
        $runData->contextAdd("backup", $sb);
    }
}
