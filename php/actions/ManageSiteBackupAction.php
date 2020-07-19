<?php
use DB\SiteBackupPeer;
use DB\SiteBackup;

class ManageSiteBackupAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));
        return true;
    }

    public function perform($r)
    {
    }

    public function requestBackupEvent($runData)
    {
        $pl = $runData->getParameterList();

        $backupSources = (bool)$pl->getParameterValue("backupSources");
        $backupFiles = (bool)$pl->getParameterValue("backupFiles");

        if (!$backupSources && !$backupFiles) {
            throw new ProcessException(_("So what do you want to backup? Choose the components."));
        }

        $site = $runData->getTemp("site");

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        SiteBackupPeer::instance()->delete($c);

        $sb = new SiteBackup();
        $sb->setSiteId($site->getSiteId());
        $sb->setBackupSource($backupSources);
        $sb->setBackupFiles($backupFiles);
        $sb->setDate(new ODate());

        $sb->save();
    }

    public function deleteBackupEvent($runData)
    {
        $site = $runData->getTemp("site");

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        SiteBackupPeer::instance()->delete($c);

        @exec('rm -r '.WIKIDOT_ROOT.'/web/files--sites/'.$site->getUnixName().'/backup');
    }
}
