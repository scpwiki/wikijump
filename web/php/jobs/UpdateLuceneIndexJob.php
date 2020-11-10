<?php
use Wikijump\Search\Lucene;

class UpdateLuceneIndexJob implements SchedulerJob
{

    public function run()
    {
        $lucene = new Lucene();
        $lucene->processQueue();
    }
}
