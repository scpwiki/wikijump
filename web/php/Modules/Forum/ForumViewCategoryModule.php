<?php

namespace Wikidot\Modules\Forum;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Wikidot\DB\ForumCategoryPeer;
use Wikidot\DB\ForumThreadPeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;

class ForumViewCategoryModule extends SmartyModule
{

    private $category;
    protected $processPage = true;

    public function render($runData)
    {
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $categoryId = $pl->getParameterValue("c");

        $parmHash = md5(serialize($pl->asArray()));

        $key = 'forumcategory_v..'.$site->getUnixName().'..'.$categoryId.'..'.$parmHash;
        $tkey = 'forumcategory_lc..'.$site->getUnixName().'..'.$categoryId; // last change timestamp
        $akey = 'forumall_lc..'.$site->getUnixName();

        $struct = Cache::get($key);
        $cacheTimestamp = $struct['timestamp'];
        $changeTimestamp = Cache::get($tkey);
        $allForumTimestamp = Cache::get($akey);
        if ($struct) {
            // check the times

            if ($changeTimestamp && $changeTimestamp <= $cacheTimestamp && $allForumTimestamp && $allForumTimestamp <= $cacheTimestamp) {
                $this->categoryName = $struct['categoryName'];
                $this->categoryId = $struct['categoryId'];
                return $struct['content'];
            }
        }

        $out = parent::render($runData);

        // and store the data now
        $struct = array();
        $now = time();
        $struct['timestamp'] = $now;
        $struct['content'] = $out;
        $struct['categoryName']=$this->categoryName;
        $struct['categoryId']=$this->categoryId;

        Cache::put($key, $struct, 864000);

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
        $categoryId = $pl->getParameterValue("c");

        $pageNumber = $pl->getParameterValue("p");
        if ($pageNumber == null || !is_numeric($pageNumber) || $pageNumber <1) {
            $pageNumber = 1;
        }

        if ($categoryId === null || !is_numeric($categoryId)) {
            throw new ProcessException(_("No forum category has been specified."), "no_category");
        }

        $sort = $pl->getParameterValue("sort");

        $c = new Criteria();
        $c->add("category_id", $categoryId);
        $c->add("site_id", $site->getSiteId());

        $category = ForumCategoryPeer::instance()->selectOne($c);

        if ($category == null || $category->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("Requested forum category does not exist."), "no_category");
        }

        $this->categoryName = $category->getName();
        $this->categoryId = $category->getCategoryId();
        // select threads...

        $perPage = 20;
        $offset = ($pageNumber - 1)*$perPage;
        $pagerData = array();
        $pagerData['current_page'] = $pageNumber;
        $pagerData['total_pages'] = ceil($category->getNumberThreads() / $perPage);

        $c = new Criteria();
        $c->add("category_id", $categoryId);
        $c->add("site_id", $site->getSiteId());
        $c->addOrderDescending("sticky");

        if ($sort == "start") {
            $c->addOrderDescending("thread_id");
        } else {
            //$c->addOrderDescending("last_post_id", "NULLS LAST"); // sorry, requires postgresql 8.3?
            $c->addOrderDescending('COALESCE(last_post_id, 0)');
            $c->addOrderDescending("thread_id");
        }
        $c->setLimit($perPage, $offset);

        $threads = ForumThreadPeer::instance()->select($c);

        $runData->contextAdd("pagerData", $pagerData);
        $runData->contextAdd("category", $category);
        $runData->contextAdd("threads", $threads);
        $runData->contextAdd("threadsCount", count($threads));
        $runData->contextAdd("sortStart", $sort=="start");
    }

    public function processPage($out, $runData)
    {
        if ($this->categoryName != null) {
            $pageTitle = $this->categoryName;
            $runData->getTemp("page")->setTitle($pageTitle); // DANGEROUS!!! DO NOT SAVE THE PAGE AFTER THIS!!!
            $out = preg_replace("/<title>(.+?)<\/title>/is", "<title>\\1 ".preg_quote_replacement(htmlspecialchars($pageTitle))."</title>", $out, 1);

            $out = preg_replace("/<div id=\"page-title\">(.*?)<\/div>/is", "<div id=\"page-title\">".htmlspecialchars($this->categoryName)."</div>", $out, 1);

            // feeds!
            $link = '/feed/forum/cp-'.$this->categoryId.'.xml';
            $out = preg_replace(
                "/<\/head>/",
                '<link rel="alternate" type="application/rss+xml" title="'._('Posts in the forum category').' &quot;'.preg_quote_replacement(htmlspecialchars($this->categoryName)).'&quot;" href="'.$link.'"/></head>',
                $out,
                1
            );
            $link = '/feed/forum/ct-'.$this->categoryId.'.xml';
            $out = preg_replace(
                "/<\/head>/",
                '<link rel="alternate" type="application/rss+xml" title="'._('Threads in the forum category').' &quot;'.preg_quote_replacement(htmlspecialchars($this->categoryName)).'&quot;" href="'.$link.'"/></head>',
                $out,
                1
            );
        }
        return $out;
    }
}
