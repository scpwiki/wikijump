<?php

namespace Wikidot\Jobs;
use Ozone\Framework\Database\Database;
use Ozone\Framework\SchedulerJob;
use Wikidot\Utils\KarmaCalculator;

/**
 * Sends email digest with unread notifications (if a user accepts this)
 */
class UpdateKarmaJob implements SchedulerJob
{

    public function run()
    {
        Database::init();
        $kc = new KarmaCalculator();
        $kc->updateAll();
    }
}
