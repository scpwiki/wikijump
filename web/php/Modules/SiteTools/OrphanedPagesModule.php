<?php

namespace Wikidot\Modules\SiteTools;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\SmartyModule;

class OrphanedPagesModule extends SmartyModule
{

    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $siteId = $site->getSiteId();

        $q = "SELECT * FROM page " .
                "WHERE page.site_id = '$siteId'" .
                "AND (SELECT count(*) FROM page_link WHERE page_link.to_page_id = page.page_id) = 0 ".
                "ORDER BY COALESCE(page.title, page.unix_name)";

        $c = new Criteria();
        $c->setExplicitQuery($q);

        $pages = [null]; // TODO run query

        $runData->contextAdd("pages", $pages);
    }
}
