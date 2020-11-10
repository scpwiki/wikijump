<?php
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
