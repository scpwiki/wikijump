<?php
use DB\ForumThreadPeer;

class ForumMiniRecentThreadsModule extends CacheableModule
{

    protected $timeOut = 300;

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        // get recent forum threads

        $pl = $runData->getParameterList();
        $limit =  $pl->getParameterValue("limit", "MODULE");

        if ($limit == null|| !is_numeric($limit) || $limit<1 || $limit>300) {
            $limit = 5;
        }

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->addOrderDescending("thread_id");
        $c->setLimit($limit);

        $threads = ForumThreadPeer::instance()->select($c);

        $runData->contextAdd("threads", $threads);
    }
}
