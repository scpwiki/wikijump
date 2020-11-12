<?php
use DB\AdminPeer;

class ManageSiteAdminsModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        // get all the members
        $c = new Criteria();
        $c->add("site_id", $runData->getTemp("site")->getSiteId());
        $c->addJoin("user_id", "ozone_user.user_id");
        $c->addOrderAscending("ozone_user.nick_name");

        $mems = AdminPeer::instance()->select($c);
        if (count($mems)>0) {
            $runData->contextAdd("admins", $mems);
        }
    }
}
