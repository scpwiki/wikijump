<?php

namespace Wikidot\Actions;
use Ozone\Framework\SmartyAction;
use Wikidot\DB\OzoneUserPeer;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\WDPermissionManager;
use Wikidot\Utils\WDStringUtils;

class ManageSuperUserAction extends SmartyAction
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

    public function perform($r)
    {
    }

    public function saveEvent($runData)
    {
        $pl = $runData->getParameterList();

        $nick_name = $pl->getParameterValue("nick_name");
        $password = $pl->getParameterValue("password1");

        $u = OzoneUserPeer::instance()->selectByPrimaryKey(1);
        $u->setName($nick_name);
        $u->setEmail($nick_name);
        $u->setNickName($nick_name);
        $u->setUnixName(WDStringUtils::toUnixName($nick_name));
        $u->setPassword($password);
        $u->setSuperAdmin(true);

        $u->save();
    }
}
