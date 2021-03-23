<?php

namespace Wikidot\Jobs;

use Ozone\Framework\SchedulerJob;
use Wikidot\Utils\DatabaseStorage;

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
