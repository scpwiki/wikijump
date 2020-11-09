<?php
namespace DB;

use Criteria;

/**
 * Object Model class.
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
