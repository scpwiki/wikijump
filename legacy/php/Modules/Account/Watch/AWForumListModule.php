<?php

namespace Wikidot\Modules\Account\Watch;




use Ozone\Framework\Database\Criteria;
use Wikidot\DB\ForumPostPeer;
use Wikidot\Utils\AccountBaseModule;

class AWForumListModule extends AccountBaseModule
{

    public function build($runData)
    {

        $user = $runData->getUser();

        $pl = $runData->getParameterList();

        $pageNumber = $pl->getParameterValue("page");
        if ($pageNumber === null) {
            $pageNumber = 1;
        }
        $limit = $pl->getParameterValue("limit");

        if ($limit == null || !is_numeric($limit) || $limit > 20) {
            $limit = 20;
        }
        $perPage = $limit;
        $offset = ($pageNumber - 1)*$perPage;
        $count = $perPage*2 + 1;

        // join the tables: watched_forum_thread, forum_thread, forum_post, site, user???. OK???

        $c = new Criteria();

        $c->addJoin("thread_id", "forum_thread.thread_id");
        $c->addJoin("thread_id", "watched_forum_thread.thread_id");
        $c->addJoin("user_id", "users.id");
        $c->add("watched_forum_thread.user_id", $user->id);
        $c->addOrderDescending("post_id");
        $c->setLimit($count, $offset);
        $posts = ForumPostPeer::instance()->select($c);

        $counted = count($posts);
        $pagerData = array();
        $pagerData['currentPage'] = $pageNumber;
        if ($counted >$perPage*2) {
            $knownPages=$pageNumber + 2;
            $pagerData['knownPages'] = $knownPages;
        } elseif ($counted >$perPage) {
            $knownPages=$pageNumber + 1;
            $pagerData['totalPages'] = $knownPages;
        } else {
            $totalPages = $pageNumber;
            $pagerData['totalPages'] = $totalPages;
        }
        $posts = array_slice($posts, 0, $perPage);

        $runData->contextAdd("pagerData", $pagerData);

        $runData->contextAdd("posts", $posts);
    }
}
