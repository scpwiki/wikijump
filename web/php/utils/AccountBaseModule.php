<?php

namespace Wikidot\Utils;

abstract use Ozone\Framework\SmartyModule;

class AccountBaseModule extends SmartyModule
{

    public function isAllowed($runData)
    {
        WDPermissionManager::instance()->hasPermission('account', $runData->getUser());
        return true;
    }
}
