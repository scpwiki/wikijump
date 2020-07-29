<?php
use DB\MemberPeer;

class MembershipByPasswordModule extends SmartyModule
{

    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $userId = $runData->getUserId();

        $settings = $site->getSettings();

        if (!$settings->getAllowMembershipByPassword() || $settings->getMembershipPassword() == null ||
            $settings->getMembershipPassword() == '') {
            $reason = "not_enabled";
            $runData->contextAdd("reason", $reason);
            return;
        }

        $reason = null;
        if (!$runData->isUserAuthenticated()) {
            $reason = "not_logged";
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

        if ($reason !== null) {
            $runData->contextAdd("reason", $reason);
        }
    }
}
