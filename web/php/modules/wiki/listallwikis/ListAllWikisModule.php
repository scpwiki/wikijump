<?php
use DB\SitePeer;

class ListAllWikisModule extends CacheableModule
{

    protected $timeOut=30;

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $categoryId = $pl->getParameterValue("c");

        $pageNumber = $pl->getParameterValue("p");
        if ($pageNumber == null || !is_numeric($pageNumber) || $pageNumber <1) {
            $pageNumber = 1;
        }

        $sort = $pl->getParameterValue("sort");

        // the criteria is: have >= 20 edits.

        // first - count them all
        //$q =

        $c = new Criteria();

        $q = "SELECT site.* FROM site WHERE  site.visible = TRUE AND site.private = FALSE AND site.deleted = FALSE AND site.site_id != 1 AND (SELECT count(*) FROM page WHERE page.site_id = site.site_id) > 15 ORDER BY site.name";

        $c->setExplicitQuery($q);

        $sites = SitePeer::instance()->select($c);

        $runData->contextAdd("sites", $sites);
    }
}
