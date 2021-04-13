<?php

namespace Wikidot\Utils;

use Ozone\Framework\SmartyModule;

abstract class AccountBaseModule extends SmartyModule
{

    public function isAllowed($runData)
    {
        WDPermissionManager::instance()->hasPermission('account', $runData->getUser());
        return true;
    }
}
