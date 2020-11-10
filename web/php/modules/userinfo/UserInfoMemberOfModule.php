<?php
use DB\MemberPeer;

class UserInfoMemberOfModule extends SmartyLocalizedModule
{

    public function build($runData)
    {

        $userId = $runData->getParameterList()->getParameterValue("user_id");

        // get all membership - criteria with join ;-) wooo!
        $c = new Criteria();
        $c->add("user_id", $userId);
        $c->addJoin("site_id", "site.site_id");
        $c->add("site.deleted", false);
        $c->addOrderAscending("site.name");

        $mems = MemberPeer::instance()->select($c);
        if (count($mems)>0) {
            $runData->contextAdd("memberships", $mems);
        }
    }
}
