<?php

namespace Wikidot\Modules\ManageSite;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\AdminPeer;
use Wikidot\Utils\ManageSiteBaseModule;

class ManageSiteAdminsModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        // get all the members
        $c = new Criteria();
        $c->add("site_id", $runData->getTemp("site")->getSiteId());
        $c->addJoin("user_id", "users.id");
        $c->addOrderAscending("users.username");

        $mems = AdminPeer::instance()->select($c);
        if (count($mems)>0) {
            $runData->contextAdd("admins", $mems);
        }
    }
}
