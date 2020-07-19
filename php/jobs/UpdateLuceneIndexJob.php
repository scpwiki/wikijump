<?php
use Wikidot\Search\Lucene;

class UpdateLuceneIndexJob implements SchedulerJob
{

    public function run()
    {
        $lucene = new Lucene();
        $lucene->processQueue();
    }
}
