<?php

namespace Wikidot\Modules\Forum;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Wikidot\DB\ForumPostPeer;

use Ozone\Framework\SmartyModule;

class ForumRecentPostsListModule extends SmartyModule
{

    public function render($runData)
    {
        $site = $runData->getTemp("site");

        $pl = $runData->getParameterList();
        $parmHash = md5(serialize($pl->asArray()));

        $key = 'forumrecentposts_v..'.$site->getSlug().'..'.$parmHash;
        $tkey = 'forumstart_lc..'.$site->getSlug(); // last change timestamp
        $akey = 'forumall_lc..'.$site->getSlug();

        $struct = Cache::get($key);
        $changeTimestamp = Cache::get($tkey);
        $allForumTimestamp = Cache::get($akey);
        if ($struct) {
            // check the times
            $cacheTimestamp = $struct['timestamp'];

            // afford 1 minute delay
            if ($changeTimestamp && $changeTimestamp <= $cacheTimestamp+60 && $allForumTimestamp && $allForumTimestamp <= $cacheTimestamp) {
                return $struct['content'];
            }
        }

        $out = parent::render($runData);

        // and store the data now
        $struct = array();
        $now = time();
        $struct['timestamp'] = $now;
        $struct['content'] = $out;

        Cache::put($key, $struct, 1000);
        if (!$changeTimestamp) {
            $changeTimestamp = $now;
            Cache::put($tkey, $changeTimestamp, 864000);
        }
        if (!$allForumTimestamp) {
            $allForumTimestamp = $now;
            Cache::put($akey, $allForumTimestamp, 864000);
        }

        return $out;
    }

    public function build($runData)
    {

        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();

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
        $c->add("forum_post.site_id", $site->getSiteId());
        $c->addJoin("thread_id", "forum_thread.thread_id");
        $c->addJoin("user_id", "users.id");
        $c->addJoin("forum_thread.category_id", "forum_category.category_id");
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
