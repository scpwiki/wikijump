<?php
use DB\PrivateUserBlockPeer;

class ASBlockedModule extends AccountBaseModule
{

    public function build($runData)
    {

        // get current blocks!

        $c = new Criteria();
        $c->add("user_id", $runData->getUserId());
        $c->addOrderDescending("block_id");

        $blocks = PrivateUserBlockPeer::instance()->select($c);
        if (count($blocks)>0) {
            $runData->contextAdd("blocks", $blocks);
        }
    }
}
