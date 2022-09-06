<?php

namespace Wikidot\Actions;
use Illuminate\Support\Facades\Hash;
use Ozone\Framework\SmartyAction;

use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\WDPermissionManager;
use Wikidot\Utils\WDStringUtils;
use Wikijump\Models\User;

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

        $u = User::find(1);
        $u->username = $nick_name;
        $u->password = Hash::make($password);
        $u->slug = WDStringUtils::toUnixName($nick_name);
        $u->save();
    }
}
