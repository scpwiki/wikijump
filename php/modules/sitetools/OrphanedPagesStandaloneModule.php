<?php
use DB\PagePeer;

class OrphanedPagesStandaloneModule extends CacheableModule
{

    protected $timeOut = 300;

    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $siteId = $site->getSiteId();

        $q = "SELECT *, count(*) AS number_links FROM page, page_link " .
                "WHERE page.site_id = '$siteId' AND page_link.to_page_id=page.page_id " .
                "GROUP BY (page.page_id) " .
                "ORDER BY COALESCE(page.title, page.unix_name)";

        $q = "SELECT * FROM page " .
                "WHERE page.site_id = '$siteId'" .
                "AND (SELECT count(*) FROM page_link WHERE page_link.to_page_id = page.page_id) = 0 ".
                "ORDER BY COALESCE(page.title, page.unix_name)";

        $c = new Criteria();
        $c->setExplicitQuery($q);

        $pages = PagePeer::instance()->select($c);

        $runData->contextAdd("pages", $pages);
    }
}
