<?php

namespace Wikidot\Modules\ManageSite;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\ModeratorPeer;
use Wikidot\Utils\ManageSiteBaseModule;

class ManageSiteModeratorsModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        $c = new Criteria();
        $c->add("site_id", $runData->getTemp("site")->getSiteId());
        $c->addJoin("user_id", "users.id");
        $c->addOrderAscending("users.username");

        $mems = ModeratorPeer::instance()->select($c);
        if (count($mems)>0) {
            $runData->contextAdd("moderators", $mems);
        }
    }
}
