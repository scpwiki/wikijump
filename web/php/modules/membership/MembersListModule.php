<?php
use DB\AdminPeer;
use DB\ModeratorPeer;
use DB\MemberPeer;

class MembersListModule extends SmartyModule
{

    public function build($runData)
    {

        $c = new Criteria();
        $c->add("site_id", $runData->getTemp("site")->getSiteId());
        $c->addJoin("user_id", "ozone_user.user_id");

        $pl = $runData->getParameterList();
        $from = $pl->getParameterValue("group", "MODULE");
        $showSince = $pl->getParameterValue("showSince", "MODULE");

        if ($showSince == "no" || $showSince == "false" || $showSince == "get lost") {
            $showSince = false;
        } else {
            $showSince = true;
        }

        if ($pl->getParameterType("from") == "MODULE") {
            $from = $pl->getParameterValue("from");
        }
        if ($from !== "admins" && $from !== "moderators") {
            $from = null;
        }

        if ($from === "admins") {
            $mems = AdminPeer::instance()->select($c);
        } elseif ($from === "moderators") {
            $mems = ModeratorPeer::instance()->select($c);
        } else {
            $mems = MemberPeer::instance()->select($c);
        }
        if (count($mems)>0) {
            $runData->contextAdd("from", $from);
            $runData->contextAdd("memberships", $mems);
            $runData->contextAdd("showSince", $showSince);
        }
    }
}
