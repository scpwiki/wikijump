<?php

namespace Wikidot\Modules\ManageSite;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\MemberApplicationPeer;
use Wikidot\Utils\ManageSiteBaseModule;

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
