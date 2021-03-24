<?php

namespace Wikidot\Modules\Login;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\GlobalProperties;

class CustomDomainScriptModule extends SmartyModule
{

    public function build($runData)
    {
        if (!$runData->getUser() && preg_match('/^([a-zA-Z0-9\-]+)\.' . GlobalProperties::$URL_DOMAIN .'$/', $_SERVER["HTTP_HOST"], $matches) !==1) {
            $runData->contextAdd("useCustomDomainScript", true);
            $runData->contextAdd("useCustomDomainScriptSecure", $_SERVER['HTTPS']);
            $runData->contextAdd("site", $runData->getTemp("site"));
        }
    }
}
