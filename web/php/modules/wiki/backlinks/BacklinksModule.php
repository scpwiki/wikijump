<?php

namespace Wikidot\Modules\Wiki\Backlinks;


use Ozone\Framework\Database\Criteria;
use Wikidot\DB\PagePeer;

use Ozone\Framework\SmartyModule;

class BacklinksModule extends SmartyModule
{
    public function build($runData)
    {

        $page = $runData->getTemp("page");
        if (!$page) {
            return;
        }
        $pageId = $page->getPageId();

        // create a very custom query
        $c = new Criteria();
        $q = "SELECT page_id, title, unix_name FROM page_link, page " .
                "WHERE page_link.to_page_id='".db_escape_string($pageId)."' " .
                "AND page_link.from_page_id=page.page_id ORDER BY COALESCE(title, unix_name)";

        $c->setExplicitQuery($q);

        $pages = PagePeer::instance()->select($c);

        $runData->contextAdd("pages", $pages);
    }
}
