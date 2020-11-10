<?php
use DB\CategoryPeer;
use DB\PagePeer;
use DB\ForumThreadPeer;

class TopRatedPagesModule extends CacheableModule2
{

    protected $keyBase = 'top_rated_pages';
    protected $timeOut = 120;
    protected $delay = 0;

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        $pl = $runData->getParameterList();
        $limit =  $pl->getParameterValue("limit", "MODULE");

        if ($limit === null|| !is_numeric($limit) || $limit<1 || $limit>300) {
            $limit = 10;
        }

        $order =$pl->getParameterValue("order");

        $minRating =$pl->getParameterValue("minRating");

        if ($minRating !== null && !is_numeric($minRating)) {
            $minRating = null;
        }

        $maxRating =$pl->getParameterValue("maxRating");

        if ($maxRating !== null && !is_numeric($maxRating)) {
            $maxRating = null;
        }

        $showComments = $pl->getParameterValue("comments", "MODULE");

        $categoryName = $pl->getParameterValue("category", "MODULE", "AMODULE");
        if ($categoryName !== null) {
            $category = CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId());
            if ($category == null) {
                throw new ProcessException(_("The category can not be found."));
            }
        }

        $c = new Criteria();
        if ($category) {
            $c->add("category_id", $category->getCategoryId());
        }
        $c->add("site_id", $site->getSiteId());

        if ($minRating!==null) {
            $c->add("rate", $minRating, '>=');
        }

        if ($maxRating!==null) {
            $c->add("rate", $maxRating, '<=');
        }

        switch ($order) {
            case 'date-created-asc':
                $c->addOrderAscending("date_created");
                break;
            case 'date-created-desc':
                $c->addOrderDescending("date_created");
                break;
            case 'rate-asc':
                $c->addOrderAscending("rate");
                break;
            case 'rating-asc':
                $c->addOrderAscending("rate");
                break;
            default:
                $c->addOrderDescending("rate");
                break;
        }

        $c->addOrderAscending("COALESCE(title, unix_name)");
        if ($limit) {
            $c->setLimit($limit);
        }

        $pages = PagePeer::instance()->select($c);

        if ($showComments) {
            foreach ($pages as &$page) {
                if ($page->getThreadId()) {
                    $thread = ForumThreadPeer::instance()->selectByPrimaryKey($page->getThreadId());
                    $noc = $thread->getNumberPosts();
                } else {
                    $noc = 0;
                }
                $page->setTemp("numberComments", $noc);
            }
        }

        $runData->contextAdd("pages", $pages);
    }
}
