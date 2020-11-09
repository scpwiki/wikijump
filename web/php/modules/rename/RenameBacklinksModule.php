<?php
use DB\PagePeer;

class RenameBacklinksModule extends SmartyModule
{
    public function build($runData)
    {

        $pageId = $runData->getParameterList()->getParameterValue("page_id");

        // create a very custom query ;-)
        $c = new Criteria();
        $q = "SELECT page_id, title, unix_name FROM page_link, page " .
                "WHERE page_link.to_page_id='".db_escape_string($pageId)."' " .
                "AND page_link.from_page_id=page.page_id ORDER BY COALESCE(title, unix_name)";

        $c->setExplicitQuery($q);

        $pages = PagePeer::instance()->select($c);

        $q = "SELECT page_id, title, unix_name FROM page, page_inclusion " .
                "WHERE page_inclusion.included_page_id='".db_escape_string($pageId)."' " .
                "AND page_inclusion.including_page_id=page.page_id ORDER BY COALESCE(title, unix_name)";

        $c = new Criteria();
        $c->setExplicitQuery($q);

        $pagesI = PagePeer::instance()->select($c);

        $merged = array();
        foreach ($pages as $key => $p) {
            $merged[$p->getPageId()] = $p;
        }
        foreach ($pagesI as $key => $p) {
            $merged[$p->getPageId()] = $p;
        }

        $runData->contextAdd("pages", $pages);
        $runData->contextAdd("pagesI", $pagesI);
        $runData->contextAdd("merged", $merged);
        $runData->contextAdd("pagesCount", count($pages));
    }
}
