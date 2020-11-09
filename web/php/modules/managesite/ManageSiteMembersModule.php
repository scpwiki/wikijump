<?php
class ManageSiteMembersModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        $site = $runData->getTemp("site");
        // get current settings:

        $settings = $site->getSettings();
        $superSettings = $site->getSuperSettings();

        $runData->contextAdd("settings", $settings);
        $runData->contextAdd("superSettings", $superSettings);
    }
}
