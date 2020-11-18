<?php
use DB\ForumCategoryPeer;
use DB\ForumPostPeer;

class ForumMiniRecentPostsModule extends CacheableModule
{

    protected $timeOut = 300;

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        $pl = $runData->getParameterList();
        $limit =  $pl->getParameterValue("limit", "MODULE");

        if ($limit == null|| !is_numeric($limit) || $limit<1 || $limit>300) {
            $limit = 5;
        }

        $categoryId = $pl->getParameterValue("categoryId", "MODULE", "AMODULE");
        if ($categoryId !== null) {
            $category = ForumCategoryPeer::instance()->selectByPrimaryKey($categoryId);
            if ($category == null || $category->getSiteId() != $site->getSiteId()) {
                throw new ProcessException(_("The category cannot be found."));
            }
        }

        // get recent forum posts

        $c = new Criteria();
        $c->add("forum_post.site_id", $site->getSiteId());
        if ($category) {
            $c->add("forum_post.category_id", $category->getCategoryId());
        }
        $c->addJoin("thread_id", "forum_thread.thread_id");
        $c->addOrderDescending("post_id");
        $c->setLimit($limit);

        $posts = ForumPostPeer::instance()->select($c);

        $runData->contextAdd("posts", $posts);
    }
}
