<?php
class ManageSiteSecureAccessModule extends ManageSiteBaseModule
{

    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $settings = $site->getSettings();

        $secureMode = $settings->getSslMode();

        $runData->contextAdd("secureMode", $secureMode);
        $runData->contextAdd("allowHttp", GlobalProperties::$ALLOW_ANY_HTTP);
    }
}
