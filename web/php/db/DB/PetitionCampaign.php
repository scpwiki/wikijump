<?php
namespace DB;

use Criteria;

/**
 * Object Model class.
 *
 */
class PetitionCampaign extends PetitionCampaignBase
{

    public function updateNumberSignatures()
    {
        $c = new Criteria();
        $c->add("campaign_id", $this->getCampaignId());
        $c->add("confirmed", true);
        $count = PetitionSignaturePeer::instance()->selectCount($c);

        $this->setNumberSignatures($count);
    }
}
