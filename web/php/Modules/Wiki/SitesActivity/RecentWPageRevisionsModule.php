<?php

namespace Wikidot\Modules\Wiki\SitesActivity;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Wikidot\DB\PagePeer;

use Ozone\Framework\SmartyModule;

class RecentWPageRevisionsModule extends SmartyModule
{

    public function render($runData)
    {
        $site = $runData->getTemp("site");
        $key = "module..0..RecentWPageRevisionsModule..".$site->getSiteId().'..'.md5(serialize($runData->getParameterList()->asArray()));

        $out = Cache::get($key);
        if (!$out) {
            $out = parent::render($runData);
            Cache::put($key, $out, 120);
        }

        return $out;
    }

    public function build($runData)
    {

        $pl = $runData->getParameterList();
        $lang = $pl->getParameterValue("lang");

        if ($lang && $lang !== "pl" && $lang !== "en") {
            $lang = null;
        }

        $c = new Criteria();
        /*
        $c->add("flag_new_site", false);
        $c->add("page.site_id", 1, '!=');
        $c->addJoin("page_id", "page.page_id");
        $c->addOrderDescending("page_revision.revision_id");
        $c->setLimit(30);

        $revs = Wikidot_DB_PageRevisionPeer::instance()->select($c);

        // check for duplications
        $revs2 = array();

        foreach($revs as $r){
            $pageId = $r->getPageId();
            if($revs2[$pageId] == null){
                $revs2[$pageId] = $r;
            }
        }

        $revs2 = array_slice($revs2, 0, 10);

        $runData->contextAdd("revisions", $revs2);

        */

        $q = "SELECT page.* FROM page, page_revision, site WHERE " .
                "page_revision.flag_new_site = FALSE ".
                "AND site.visible = TRUE AND site.private = FALSE
				AND site.deleted = FALSE " ;

        if ($lang) {
            $q.= "AND site.language = '".db_escape_string($lang)."' ";
        }

        $q.=        "AND page.site_id != 1".
                "AND page.revision_id = page_revision.revision_id ".
                "AND page.site_id = site.site_id " .
                "ORDER BY page.revision_id DESC LIMIT 10";
        $c->setExplicitQuery($q);

        $pages = PagePeer::instance()->select($c);
        $runData->contextAdd("pages", $pages);
    }
}
