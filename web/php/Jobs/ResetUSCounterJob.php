<?php

namespace Wikidot\Jobs;
use Ozone\Framework\SchedulerJob;
use Ozone\Framework\UniqueStrings;

/**
 * Resets unique strings counters.
 *
 */
class ResetUSCounterJob implements SchedulerJob
{

    public function run()
    {
        UniqueStrings::resetCounter();
    }
}
