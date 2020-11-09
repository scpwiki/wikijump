<?php
class ManageSiteLetUsersInviteModule extends ManageSiteBaseModule
{

    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $settings = $site->getSettings();

        $runData->contextAdd("enabled", $settings->getAllowMembersInvite());
    }
}
