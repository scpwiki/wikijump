<?php
use DB\MemberApplicationPeer;

class ManageSiteMembersApplicationsModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        // find all current applications
        $c = new Criteria();
        $c->add("site_id", $runData->getTemp("site")->getSiteId());
        $c->add("status", "pending");
        $c->addOrderDescending("application_id");

        $applications = MemberApplicationPeer::instance()->select($c);
        $runData->contextAdd("applications", $applications);
    }
}
