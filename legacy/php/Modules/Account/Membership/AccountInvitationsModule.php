<?php

namespace Wikidot\Modules\Account\Membership;




use Ozone\Framework\Database\Criteria;
use Wikidot\DB\MemberInvitationPeer;
use Wikidot\Utils\AccountBaseModule;

class AccountInvitationsModule extends AccountBaseModule
{

    public function build($runData)
    {

        // just get invitations
        $userId = $runData->getUserId();
        $c = new Criteria();
        $c->add("user_id", $userId);
        $c->addOrderDescending("invitation_id");

        $invs = MemberInvitationPeer::instance()->select($c);

        if (count($invs)>0) {
            $runData->contextAdd("invitations", $invs);
        }
    }
}
