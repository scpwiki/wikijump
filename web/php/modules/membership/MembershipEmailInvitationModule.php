<?php
use DB\EmailInvitationPeer;
use DB\SitePeer;
use DB\OzoneUserPeer;

class MembershipEmailInvitationModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $user = $runData->getUser();
        $hash = $pl->getParameterValue("hash");

        // get the invitation entry (if any)

        $c = new Criteria();
        $c->add("hash", $hash);
        $c->add("accepted", false);

        $inv = EmailInvitationPeer::instance()->selectOne($c);

        $runData->contextAdd("user", $user);

        if (!$inv) {
            //sorry, no invitation
            return;
        }

        $site = SitePeer::instance()->selectByPrimaryKey($inv->getSiteId());

        $sender = OzoneUserPeer::instance()->selectByPrimaryKey($inv->getUserId());
        $runData->contextAdd("sender", $sender);
        $runData->contextAdd("site", $site);
        $runData->contextAdd("invitation", $inv);
        $runData->contextAdd("hash", $hash);
    }
}
