<?php
use DB\EmailInvitationPeer;

class SentMemberInvitationsModule extends SmartyModule
{

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        $user = $runData->getUser();

        if (!$user) {
            $runData->setModuleTemplate("misc/AskToLoginModule");
            return;
        }

        // now get the ivitations!
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("user_id", $user->getUserId());
        $c->addOrderDescending("invitation_id");

        $invitations = EmailInvitationPeer::instance()->select($c);

        $runData->contextAdd("invitations", $invitations);
    }
}
