<?php

namespace Wikidot\Modules\UserInfo;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\ForumPostPeer;
use Wikidot\Utils\SmartyLocalizedModule;

class UserRecentPostsListModule extends SmartyLocalizedModule
{

    public function build($runData)
    {

        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();

        $userId = $pl->getParameterValue("userId");

        if ($runData->getUser() && $userId == $runData->getUser()->id) {
            $own = true;
        }

        $categoryId = $pl->getParameterValue("categoryId");
        $limit = $pl->getParameterValue("limit");

        if ($limit == null || !is_numeric($limit) || $limit > 20) {
            $limit = 20;
        }

        $pageNumber = $pl->getParameterValue("page");
        $op = $pl->getParameterValue("options");

        if ($pageNumber === null) {
            $pageNumber = 1;
        }
        $perPage = $limit;
        $offset = ($pageNumber - 1)*$perPage;
        $count = $perPage*2 + 1;

        $c = new Criteria();
        if ($categoryId !== null && is_numeric($categoryId)) {
            $c->add("forum_thread.category_id", $categoryId);
        }
        $c->add("forum_post.user_id", $userId);
        if (!$own) {
            $c->add("site.private", false);
        }
        $c->addJoin("thread_id", "forum_thread.thread_id");
        $c->addJoin("user_id", "users.id");
        $c->addJoin("forum_post.site_id", "site.site_id");
        $c->add("site.deleted", false);
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
