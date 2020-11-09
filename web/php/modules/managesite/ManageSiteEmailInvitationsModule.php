<?php
class ManageSiteEmailInvitationsModule extends ManageSiteBaseModule
{

    protected $processPage = true;

    public function build($runData)
    {

        $site = $runData->getTemp("site");
        $runData->contextAdd("site", $site);
        $runData->contextAdd("settings", $site->getSettings());

        $runData->contextAdd("user", $runData->getUser());
        $runData->contextAdd("profile", $runData->getUser()->getProfile());
    }
}
