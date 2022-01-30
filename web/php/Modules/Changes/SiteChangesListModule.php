<?php

namespace Wikidot\Modules\Changes;
use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\JSONService;
use Ozone\Framework\Ozone;
use Wikidot\DB\PageRevisionPeer;

use Ozone\Framework\SmartyModule;

class SiteChangesListModule extends SmartyModule
{

    public function render($runData)
    {
        $site = $runData->getTemp("site");

        $pl = $runData->getParameterList();
        $parmHash = md5(serialize($pl->asArray()));

        $key = 'siterecentrevisions_v..'.$site->getSlug().'..'.$parmHash;
        $tkey = 'siterevisions_lc..'.$site->getSiteId(); // last change timestamp

        $struct = Cache::get($key);
        $changeTimestamp = Cache::get($tkey);

        if ($struct) {
            // check the times
            $cacheTimestamp = $struct['timestamp'];

            // afford 1 minute delay
            if ($changeTimestamp && $changeTimestamp <= $cacheTimestamp) {
                return $struct['content'];
            }
        }

        $out = parent::render($runData);

        // and store the data now
        $struct = array();
        $now = time();
        $struct['timestamp'] = $now;
        $struct['content'] = $out;

        Cache::put($key, $struct, 600);
        if (!$changeTimestamp) {
            $changeTimestamp = $now;
            Cache::put($tkey, $changeTimestamp, 3600);
        }

        return $out;
    }

    public function build($runData)
    {
        // select recent revisions...

        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        // get options
        $pageNumber = $pl->getParameterValue("page");
        $op = $pl->getParameterValue("options");

        if ($pageNumber === null) {
            $pageNumber = 1;
        }

        if ($op) {
            $o = json_decode($op, true);
            if (is_countable($o) == false) {
                $o['all'] == true;
            }
        }

        $perPage = $pl->getParameterValue("perpage");
        if ($perPage == null) {
            $perPage=20;
        }

        $offset = ($pageNumber - 1)*$perPage;
        $count = $perPage*2 + 1;

        $c = new Criteria();
        $c->add("page_revision.site_id", $site->getSiteId());

        if (!$o['all'] && is_countable($o)) {
            $c2 = new Criteria();
            if ($o['new']) {
                $c2->addOr("flag_new", true);
            }
            if ($o['source']) {
                $c2->addOr("flag_text", true);
            }
            if ($o['title']) {
                $c2->addOr("flag_title", true);
            }
            if ($o['move']) {
                $c2->addOr("flag_rename", true);
            }
            if ($o['meta']) {
                $c2->addOr("flag_meta", true);
            }
            if ($o['files']) {
                $c2->addOr("flag_file", true);
            }
            $c->addCriteriaAnd($c2);
        }

        $categoryId = $pl->getParameterValue("categoryId");

        if ($categoryId && is_numeric($categoryId)) {
            $c->add("page.category_id", $categoryId);
        }

        $c->addJoin("page_id", "page.page_id");
        $c->addJoin("user_id", "users.id");
        $c->addOrderDescending("page_revision.revision_id");
        $c->setLimit($count, $offset);
        $revisions = PageRevisionPeer::instance()->select($c);

        $counted = count($revisions);
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

        $revisions = array_slice($revisions, 0, $perPage);
        $runData->contextAdd("pagerData", $pagerData);
        $runData->contextAdd("revisions", $revisions);
        $runData->contextAdd("revisionsCount", count($revisions));
    }
}
