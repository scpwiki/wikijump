<?php
use DB\ForumThreadPeer;

class ForumMiniActiveThreadsModule extends CacheableModule
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

        $date = new ODate();
        $date->addSeconds(-60*60*24*7); // 7 days

        $q = "SELECT forum_thread.thread_id, count(*) AS count FROM forum_thread, forum_post " .
                "WHERE forum_thread.site_id='".$site->getSiteId()."' " .
                "AND forum_thread.thread_id = forum_post.thread_id " .
                "AND forum_post.date_posted > '". $date->getDate()."' " .
                "GROUP BY forum_thread.thread_id ORDER BY count DESC LIMIT ".db_escape_string($limit) ;

        $c = new Criteria();
        $c->setExplicitQuery($q);

        $threads = ForumThreadPeer::instance()->select($c);

        foreach ($threads as &$thread) {
            $thread = ForumThreadPeer::instance()->selectByPrimaryKey($thread->getThreadId());
        }

        $runData->contextAdd("threads", $threads);
    }
}
