<?php
use DB\ModeratorPeer;

class ManageSiteModeratorsModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        $c = new Criteria();
        $c->add("site_id", $runData->getTemp("site")->getSiteId());
        $c->addJoin("user_id", "ozone_user.user_id");
        $c->addOrderAscending("ozone_user.nick_name");

        $mems = ModeratorPeer::instance()->select($c);
        if (count($mems)>0) {
            $runData->contextAdd("moderators", $mems);
        }
    }
}
