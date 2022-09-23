<?php

namespace Wikidot\Modules\ManageSite;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\MemberPeer;
use Wikidot\Utils\ManageSiteBaseModule;

class ManageSiteMembersListModule extends ManageSiteBaseModule
{

    public function build($runData)
    {
        $c = new Criteria();
        $c->add("site_id", $runData->getTemp("site")->getSiteId());
        $c->addJoin("user_id", "users.id");

        $mems = MemberPeer::instance()->select($c);
        if (count($mems)>0) {
            $runData->contextAdd("memberships", $mems);
        }
    }
}
