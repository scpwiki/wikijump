<?php

namespace Wikidot\Actions;
use Ozone\Framework\Database\Criteria;
use Wikidot\DB\PageEditLockPeer;

class PageLockAction
{

    public function perform($runData)
    {
    }

    /**
     * Simply removes page edit lock from a page.
     */
    public function removePageEditLockEvent($runData)
    {
        $pageId =  $runData->getParameterList()->getParameterValue("page_id");
        $c = new Criteria();
        $c->add("page_id", $pageId);
        PageEditLockPeer::instance()->delete($c);
    }
}
