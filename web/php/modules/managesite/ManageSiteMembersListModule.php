<?php

namespace Wikidot\Modules\ManageSite;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\MemberPeer;
use Wikidot\Utils\ManageSiteBaseModule;

class ManageSiteMembersListModule extends ManageSiteBaseModule
{

    public function build($runData)
    {
        // get all the members
        /*
        $c = new Criteria();
        $c->setExplicitFrom("ozone_user, member");
        $c->add("member.site_id", $runData->getTemp("site")->getSiteId());
        $c->add("member.user_id", "ozone_user.user_id", "=", false);
        $c->addOrderAscending("nick_name");

        $members = DB_OzoneUserPeer::instance()->select($c);

        $runData->contextAdd("members", $members);

        */
        $c = new Criteria();
        $c->add("site_id", $runData->getTemp("site")->getSiteId());
        $c->addJoin("user_id", "ozone_user.user_id");

        $mems = MemberPeer::instance()->select($c);
        if (count($mems)>0) {
            $runData->contextAdd("memberships", $mems);
        }
    }
}
