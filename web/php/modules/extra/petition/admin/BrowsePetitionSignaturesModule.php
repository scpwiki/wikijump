<?php
use DB\PetitionCampaignPeer;
use DB\PetitionSignaturePeer;

class BrowsePetitionSignaturesModule extends SmartyModule
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
            throw new ProcessException(_("The campaign can not be found."));
        }

        // get signatures!

        $c = new Criteria();
        $c->add("campaign_id", $camp->getCampaignId());
        $c->add("confirmed", true);
        $c->addOrderAscending("signature_id");
        $signatures = PetitionSignaturePeer::instance()->select($c);

        $runData->contextAdd("signatures", $signatures);
        $runData->contextAdd("campaign", $camp);
    }
}
