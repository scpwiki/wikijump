<?php

namespace Wikidot\Modules\PageRate;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\SmartyModule;
use Wikidot\DB\PageRateVotePeer;
use Wikijump\Services\Deepwell\Models\Page;

class WhoRatedPageModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("pageId");

        $page = Page::findIdOnly($pageId);

        $c = new Criteria();
        $c->add("page_id", $page->getPageId());
        $c->addJoin("user_id", "users.id");
        $c->addOrderAscending("users.username");

        $rates = PageRateVotePeer::instance()->select($c);

        $runData->contextAdd("rates", $rates);
    }
}
