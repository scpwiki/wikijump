<?php
use DB\PetitionCampaignPeer;

class ViewPetitionCampaignModule extends SmartyModule
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
        // get the petition campaign...
        $campaignId = $pl->getParameterValue("campaignId");

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("deleted", false);
        $c->add("campaign_id", $campaignId);

        $camp = PetitionCampaignPeer::instance()->selectOne($c);

        if (!$camp) {
            throw new ProcessException(_("The campaign cannot be found."));
        }

        $runData->contextAdd("campaign", $camp);

        // get all campaigns
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("deleted", false);
        $camps = PetitionCampaignPeer::instance()->select($c);

        $runData->contextAdd("campaigns", $camps);
        $runData->contextAdd("campaignsCount", count($camps));
    }
}
