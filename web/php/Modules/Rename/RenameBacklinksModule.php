<?php

namespace Wikidot\Modules\Rename;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\SmartyModule;
use Wikijump\Services\Deepwell\Models\Page;

class RenameBacklinksModule extends SmartyModule
{
    public function build($runData)
    {

        $pageId = $runData->getParameterList()->getParameterValue("page_id");

        // create a very custom query
        $c = new Criteria();
        $q = "SELECT page_id, title, unix_name FROM page_link, page " .
                "WHERE page_link.to_page_id='".db_escape_string($pageId)."' " .
                "AND page_link.from_page_id=page.page_id ORDER BY COALESCE(title, unix_name)";

        $c->setExplicitQuery($q);

        $pages = [null]; // TODO run query

        $q = "SELECT page_id, title, unix_name FROM page, page_inclusion " .
                "WHERE page_inclusion.included_page_id='".db_escape_string($pageId)."' " .
                "AND page_inclusion.including_page_id=page.page_id ORDER BY COALESCE(title, unix_name)";

        $c = new Criteria();
        $c->setExplicitQuery($q);

        $pagesI = [null]; // TODO run query

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
