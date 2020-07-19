<?php
use DB\PetitionCampaignPeer;

class PetitionAdminModule extends SmartyModule
{

    public function isAllowed($runData)
    {
        WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));
        return true;
    }

    public function build($runData)
    {

        $pl = $runData->getParameterList();

        $site = $runData->getTemp("site");

        // get current campaigns

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("deleted", false);
        $c->addOrderAscending("campaign_id");

        $camps = PetitionCampaignPeer::instance()->select($c);

        $runData->contextAdd("campaigns", $camps);

        $withoutBox = (bool) $pl->getParameterValue("withoutBox");
        $runData->contextAdd("withoutBox", $withoutBox);
    }
}
