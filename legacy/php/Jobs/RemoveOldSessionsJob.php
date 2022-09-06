<?php

namespace Wikidot\Jobs;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\SchedulerJob;
use Wikidot\DB\OzoneSessionPeer;

/**
 * Removes old (expired) sessions from the database.
 *
 */
class RemoveOldSessionsJob implements SchedulerJob
{

    public function run()
    {
        $c = new Criteria();
        $c->add("infinite", false);
        $date = currentDateUTC();
        $date->subtractSeconds(3600);
        $c->add("last_accessed", $date, "<");

        OzoneSessionPeer::instance()->delete($c);
    }
}
