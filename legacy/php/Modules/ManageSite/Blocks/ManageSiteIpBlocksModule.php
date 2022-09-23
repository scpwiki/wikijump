<?php

namespace Wikidot\Modules\ManageSite\Blocks;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\IpBlockPeer;
use Wikidot\Utils\ManageSiteBaseModule;

class ManageSiteIpBlocksModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        // get current blocks!
        $site = $runData->getTemp("site");

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->addOrderDescending("block_id");

        $blocks = IpBlockPeer::instance()->select($c);
        if (count($blocks)>0) {
            $runData->contextAdd("blocks", $blocks);
        }
    }
}
