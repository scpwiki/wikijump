<?php
use DB\AdminPeer;

class AccountAdminOfModule extends AccountBaseModule
{

    public function build($runData)
    {

        $userId = $runData->getUserId();

        // get all membership - criteria with join - wooo!
        $c = new Criteria();
        $c->add("user_id", $userId);
        $c->addJoin("site_id", "site.site_id");
        $c->add("site.deleted", false);

        $mems = AdminPeer::instance()->select($c);
        if (count($mems)>0) {
            $runData->contextAdd("admins", $mems);
        }
    }
}
