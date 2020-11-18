<?php
use DB\MemberInvitationPeer;

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
