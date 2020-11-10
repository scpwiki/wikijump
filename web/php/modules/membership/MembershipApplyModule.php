<?php
use DB\MemberPeer;
use DB\MemberApplicationPeer;

class MembershipApplyModule extends SmartyModule
{

    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $userId = $runData->getUserId();

        $reason = null;
        if (!$runData->isUserAuthenticated()) {
            $reason = "not_logged";
        }

        $settings = $site->getSettings();

        if (!$settings->getAllowMembershipByApply()) {
            $reason = "not_enabled";
            $runData->contextAdd("reason", $reason);
            return;
        }

        // check if not a member already
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("user_id", $userId);
        $a = MemberPeer::instance()->selectOne($c);
        if ($a != null) {
            $reason = "already_member";
            $runData->contextAdd("reason", $reason);
            return;
        }

        // see if there is already an application...
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("user_id", $userId);
        $a = MemberApplicationPeer::instance()->selectOne($c);
        if ($a != null) {
            $reason = "already_applied";
            $runData->contextAdd("reason", $reason);
            return;
        }

        if ($reason !== null) {
            $runData->contextAdd("reason", $reason);
        }
    }
}
