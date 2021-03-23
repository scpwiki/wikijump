<?php

namespace Wikidot\DB;


use Ozone\Framework\Database\Criteria;

/**
 * Object Model Class.
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
