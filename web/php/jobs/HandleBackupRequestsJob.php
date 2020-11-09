<?php
use DB\SiteBackupPeer;

/**
 * Periodically creates downloadable site backups (on request).
 *
 */
class HandleBackupRequestsJob implements SchedulerJob
{

    public function run()
    {

        // check!
        $c = new Criteria();
        $c->add("status", null);
        $c->addOrderDescending("backup_id");

        $sb = SiteBackupPeer::instance()->selectOne($c); // select only one!

        if (!$sb) {
            return;
        }

        $db = Database::connection();
        $sb->setStatus("started");
        $sb->save();

        $db->begin();

        try {
            $b = new Backuper();
            $b->setConfig($sb);
            $b->backup();

            // check

            $sb->setStatus("completed");
            $sb->setDate(new ODate());
            $sb->setRand($b->getRand());

            $sb->save();
        } catch (Exception $e) {
            $sb->setStatus("failed");
            $sb->save();
        }
        $db->commit();
    }
}
