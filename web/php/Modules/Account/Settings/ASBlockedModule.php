<?php

namespace Wikidot\Modules\Account\Settings;




use Ozone\Framework\Database\Criteria;
use Wikidot\DB\PrivateUserBlockPeer;
use Wikidot\Utils\AccountBaseModule;

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
