<?php

namespace Wikidot\Modules\Backlinks;
use Ozone\Framework\Database\Criteria;
use Wikidot\DB\PagePeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;

class BacklinksModule extends SmartyModule
{
    public function build($runData)
    {

        $pageId = $runData->getParameterList()->getParameterValue("pageId");

        if (!$pageId || !is_numeric($pageId)) {
            throw new ProcessException(_("The page cannot be found or does not exist."), "no_page");
        }

        // create a very custom query
        $c = new Criteria();
        $q = "SELECT page_id, title, unix_name FROM page_link, page " .
                "WHERE page_link.to_page_id='".db_escape_string($pageId)."' " .
                "AND page_link.from_page_id=page.page_id ORDER BY COALESCE(title, unix_name)";

        $c->setExplicitQuery($q);

        $pages = PagePeer::instance()->select($c);

        $q = "SELECT page_id, title, unix_name FROM page, page_inclusion " .
                "WHERE page_inclusion.included_page_id='".db_escape_string($pageId)."' " .
                "AND page_inclusion.including_page_id=page.page_id ORDER BY COALESCE(title, unix_name)";

        $c->setExplicitQuery($q);

        $pagesI = PagePeer::instance()->select($c);

        $runData->contextAdd("pagesI", $pagesI);

        $runData->contextAdd("pages", $pages);
        $runData->contextAdd("pagesCount", count($pages));
    }
}
