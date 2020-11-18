<?php
/**
 * Cleans database storage for temporary objects.
 *
 */
class OutdateDatabaseStorageJob implements SchedulerJob
{

    public function run()
    {
        DatabaseStorage::instance()->clean();
    }
}
