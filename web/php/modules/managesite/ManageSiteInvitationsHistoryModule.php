<?php
use DB\EmailInvitationPeer;

class ManageSiteInvitationsHistoryModule extends ManageSiteBaseModule
{

    protected $processPage = true;

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        $showAll = (bool) $runData->getParameterList()->getParameterValue("showAll");
        // get  invitations
        $c = new Criteria();

        if (!$showAll) {
            $q = "SELECT * FROM email_invitation, admin " .
                "WHERE admin.site_id='".$site->getSiteId()."' " .
                        "AND email_invitation.site_id='".$site->getSiteId()."' " .
                        "AND admin.user_id = email_invitation.user_id ORDER BY invitation_id DESC";
            $c->setExplicitQuery($q);
        } else {
            $c->add("site_id", $site->getSiteId());
            $c->addOrderDescending("invitation_id");
        }

        $invitations = EmailInvitationPeer::instance()->select($c);

        $runData->contextAdd("invitations", $invitations);
        $runData->contextAdd("showAll", $showAll);
    }
}
