<?php
use DB\OzoneUserPeer;
use DB\SitePeer;
use DB\PagePeer;

class UserInfoProfileModule extends SmartyLocalizedModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $userId = $pl->getParameterValue("user_id");

        $user = OzoneUserPeer::instance()->selectByPrimaryKey($userId);
        $runData->contextAdd("user", $user);

        $avatarUri = '/common--images/avatars/'.floor($userId/1000).'/'.$userId.'/a48.png';
        $runData->contextAdd("avatarUri", $avatarUri);

        // get profile page to include
        $pageName = "profile:".$user->getUnixName();

        $c = new Criteria();
        $c->add("unix_name", "profiles");
        $site = SitePeer::instance()->selectOne($c);

        $page = PagePeer::instance()->selectByName($site->getSiteId(), $pageName);

        if ($page !== null) {
            $compiled = $page->getCompiled();
            $runData->contextAdd("profileContent", $compiled);
            $runData->contextAdd("wikiPage", $page);
        }
        $runData->contextAdd('karmaLevel', $user->getKarmaLevel());
    }
}
