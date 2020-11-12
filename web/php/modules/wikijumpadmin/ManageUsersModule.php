<?php
use DB\OzoneUserPeer;

class ManageUsersModule extends SmartyModule
{

    public function isAllowed($runData)
    {
        if ($runData->getTemp("site")->getSiteId() != 1) {
            throw new WDPermissionException("No permission");
        }
        WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));

        return true;
    }

    public function build($runData)
    {

        $users = array();
        $c = new Criteria();
        $c->add('user_id', '1', '>');

        foreach (OzoneUserPeer::instance()->select($c) as $user) {
            $admin = WDPermissionManager::hasPermission('manage_site', $user, 1) ? 1 : 0;
            $mod = WDPermissionManager::hasPermission('moderate_site', $user, 1) ? 1 : 0;

            $users[] = array(
                "nick_name" => $user->getNickName(),
                "user_id" => $user->getUserId(),
                "mod" => $mod,
                "admin" => $admin,
            );
        }
        for ($i = 0; $i < 5; $i++) {
            $users[] = array("user_id" => "new$i");
        }
        $runData->contextAdd("users", $users);
    }
}
