<?php

namespace Wikidot\Jobs;
use Ozone\Framework\SchedulerJob;
use Wikidot\Search\Lucene;

class UpdateLuceneIndexJob implements SchedulerJob
{

    public function run()
    {
        $lucene = new Lucene();
        $lucene->processQueue();
    }
}
