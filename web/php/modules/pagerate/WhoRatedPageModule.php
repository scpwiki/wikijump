<?php
use DB\PagePeer;
use DB\PageRateVotePeer;

class WhoRatedPageModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("pageId");

        $page = PagePeer::instance()->selectByPrimaryKey($pageId);

        $c = new Criteria();
        $c->add("page_id", $page->getPageId());
        $c->addJoin("user_id", "ozone_user.user_id");
        $c->addOrderAscending("ozone_user.nick_name");

        $rates = PageRateVotePeer::instance()->select($c);

        $runData->contextAdd("rates", $rates);
    }
}
