<?php
use DB\OzoneUserPeer;
use DB\MemberPeer;
use DB\MembershipLinkPeer;

class WhoInvitedResultsModule extends SmartyModule
{

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        $pl = $runData->getParameterList();

        $userId = $pl->getParameterValue("userId");
        $user = OzoneUserPeer::instance()->selectByPrimaryKey($userId);

        if (!$user) {
            throw new ProcessException(_("Invalid user."));
        }
        $c = new Criteria();
        $c->add("user_id", $userId);
        $c->add("site_id", $site->getSiteId());
        $mem = MemberPeer::instance()->selectOne($c);

        if (!$mem) {
            throw new ProcessException(_("The user is not a Member of this Wiki."));
        }

        $link = MembershipLinkPeer::instance()->selectByUserId($site->getSiteId(), $userId);
        if (!$link) {
            $runData->contextAdd("noData", true);
        } else {
            $chain = array();
            $chain[] = array('user' => $user, 'link' => $link);
            if ($link->getByUserId()) {
                do {
                    // get "parent"
                    // get link for the user
                    $user = OzoneUserPeer::instance()->selectByPrimaryKey($link->getByUserId());
                    $link = MembershipLinkPeer::instance()->selectByUserId($site->getSiteId(), $user->getUserId());
                    $chain[] = array('user' => $user, 'link' => $link);
                } while ($user && $link && $link->getByUserId());
            }
            $runData->contextAdd("chain", array_reverse($chain));
        }
    }
}
