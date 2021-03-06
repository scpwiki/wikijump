<?php
use DB\OzoneUserPeer;
use DB\MemberPeer;
use DB\AdminPeer;

class UserInfoWinModule extends SmartyModule
{

    public function build($runData)
    {

        $pl = $runData->getParameterList();
        $userId = $pl->getParameterValue("user_id");

        $user = OzoneUserPeer::instance()->selectByPrimaryKey($userId);
        $avatarUri = '/common--images/avatars/'.floor($userId/1000).'/'.$userId.'/a48.png';
        $runData->contextAdd("user", $user);
        $runData->contextAdd("avatarUri", $avatarUri);

        // find the possible role in this site

        $site = $runData->getTemp("site");
        $siteId = $site->getSiteId();

        $c = new Criteria();
        $c->add("user_id", $userId);
        $c->add("site_id", $siteId);
        $mem = MemberPeer::instance()->selectOne($c);
        if ($mem != null) {
            $runData->contextAdd("member", $mem);
            // also check for other roles: admin & moderator
            if (AdminPeer::instance()->selectOne($c) != null) {
                $runData->contextAdd("role", "admin");
            } elseif (AdminPeer::instance()->selectOne($c) != null) {
                $runData->contextAdd("role", "moderator");
            }
        }

        $runData->contextAdd("uu", $runData->getUser());
        $runData->contextAdd('karmaLevel', $user->getKarmaLevel());
    }
}
