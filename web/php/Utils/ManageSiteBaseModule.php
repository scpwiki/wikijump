<?php

namespace Wikidot\Utils;

abstract use Ozone\Framework\SmartyModule;

class ManageSiteBaseModule extends SmartyModule
{

    public function isAllowed($runData)
    {
        WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));
        return true;
    }
}
