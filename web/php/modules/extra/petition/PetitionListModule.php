<?php
use DB\PetitionCampaignPeer;
use DB\PetitionSignaturePeer;

class PetitionListModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");
        // get the petition campaign...
        $campaignId = $pl->getParameterValue("id");

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("deleted", false);
        $c->add("identifier", $campaignId);

        $camp = PetitionCampaignPeer::instance()->selectOne($c);

        if (!$camp) {
            throw new ProcessException(_("The campaign cannot be found."));
        }

        // get signatures!

        $limit = $pl->getParameterValue("limit");
        if ($limit === null || !is_numeric($limit)) {
            $limit = 50;
        }

        $c = new Criteria();
        $c->add("campaign_id", $camp->getCampaignId());
        $c->add("confirmed", true);
        $c->addOrderDescending("signature_id");
        if ($limit > 0) {
            $c->setLimit($limit);
        }
        $signatures = PetitionSignaturePeer::instance()->select($c);

        $runData->contextAdd("signatures", $signatures);
        $runData->contextAdd("campaign", $camp);
    }
}
