<?php

namespace Wikidot\DB;


use Ozone\Framework\Database\Criteria;

/**
 * Object Model Class.
 *
 */
class MembershipLinkPeer extends MembershipLinkPeerBase
{

    public function selectByUserId($siteId, $userId)
    {
        $c = new Criteria();
        $c->add("site_id", $siteId);
        $c->add("user_id", $userId);
        return $this->selectOne($c);
    }
}
