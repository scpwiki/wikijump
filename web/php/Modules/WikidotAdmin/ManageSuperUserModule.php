<?php

namespace Wikidot\Modules\WikidotAdmin;



use Ozone\Framework\SmartyModule;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\WDPermissionManager;
use Wikijump\Models\User;

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

        $o = User::find(1);
        $u = array(
            "nick_name" => $o->username(),
        );
        $runData->contextAdd("user", $u);

        if ($key = $pl->getParameterValue("key")) {
            $runData->contextAdd("key", $key);
        }
    }
}
