<?php
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
