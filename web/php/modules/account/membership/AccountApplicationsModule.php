<?php
use DB\MemberApplicationPeer;

class AccountApplicationsModule extends AccountBaseModule
{

    public function build($runData)
    {

        // get applications by a user
        $userId = $runData->getUserId();

        // get all applications - criteria with join ;-) wooo!
        $c = new Criteria();
        $c->add("user_id", $userId);
        $c->addJoin("site_id", "site.site_id");
        $c->add("site.deleted", false);

        $apps = MemberApplicationPeer::instance()->select($c);
        if (count($apps)>0) {
            $runData->contextAdd("applications", $apps);
        }
    }
}
