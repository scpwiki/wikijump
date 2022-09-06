<?php

namespace Wikidot\Modules\Forum;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Wikidot\DB\ForumGroupPeer;
use Wikidot\DB\ForumCategoryPeer;

use Ozone\Framework\SmartyModule;

class ForumStartModule extends SmartyModule
{

    protected $processPage = true;

    public function render($runData)
    {
        $site = $runData->getTemp("site");

        $pl = $runData->getParameterList();
        $parmHash = md5(serialize($pl->asArray()));

        $key = 'forumstart_v..'.$site->getSlug().'..'.$parmHash;
        $tkey = 'forumstart_lc..'.$site->getSlug(); // last change timestamp
        $akey = 'forumall_lc..'.$site->getSlug();

        $struct = Cache::get($key);
        $cacheTimestamp = $struct['timestamp'];
        $changeTimestamp = Cache::get($tkey);
        $allForumTimestamp = Cache::get($akey);
        if ($struct) {
            // check the times

            if ($changeTimestamp && $changeTimestamp <= $cacheTimestamp && $allForumTimestamp && $allForumTimestamp <= $cacheTimestamp) {
                return $struct['content'];
            }
        }

        $out = parent::render($runData);

        // and store the data now
        $struct = array();
        $now = time();
        $struct['timestamp'] = $now;
        $struct['content'] = $out;

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

        $pl = $runData->getParameterList();

        $site = $runData->getTemp("site");
        // get groups and categories

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        if (!$pl->getParameterValue("hidden")) {
            $c->add("visible", true);
            $runData->contextAdd("hidden", true);
        }
        $c->addOrderAscending("sort_index");

        $groups = ForumGroupPeer::instance()->select($c);

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->addOrderAscending("sort_index");

        $categories = ForumCategoryPeer::instance()->select($c);

        // now mangle the categories and put into array
        // - in order to avoid several queries

        $cats = array();
        foreach ($categories as $category) {
            $cats[$category->getGroupId()][] = $category;
        }

        $runData->contextAdd("groups", $groups);
        $runData->contextAdd("catarray", $cats);
    }

    public function processPage($out, $runData)
    {
        $site = $runData->getTemp("site");
        $link = '/feed/forum/threads.xml';
        $title =  $site->getName()." - "._("new forum threads");
        $out = preg_replace(
            "/<\/head>/",
            '<link rel="alternate" type="application/rss+xml" title="'.preg_quote_replacement(htmlspecialchars($title)).'" href="'.$link.'"/></head>',
            $out,
            1
        );

        $link = '/feed/forum/posts.xml';
        $title =  $site->getName()." - new forum posts";
        $out = preg_replace(
            "/<\/head>/",
            '<link rel="alternate" type="application/rss+xml" title="'.preg_quote_replacement(htmlspecialchars($title)).'" href="'.$link.'"/></head>',
            $out,
            1
        );

        return $out;
    }
}
