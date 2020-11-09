<?php
use DB\OzoneUserPeer;

class ManageSuperUserModule extends SmartyModule
{

    public function isAllowed($runData)
    {
        $pl = $runData->getParameterList();
        if ($key = $pl->getParameterValue("key")) {
            if (GlobalProperties::$SECRET_MANAGE_SUPERADMIN == $key) {
                return true;
            }
        }

        WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));

        return true;
    }

    public function build($runData)
    {

        $pl = $runData->getParameterList();

        $o = OzoneUserPeer::instance()->selectByPrimaryKey(1);
        $u = array(
            "nick_name" => $o->getNickName(),
        );
        $runData->contextAdd("user", $u);

        if ($key = $pl->getParameterValue("key")) {
            $runData->contextAdd("key", $key);
        }
    }
}
