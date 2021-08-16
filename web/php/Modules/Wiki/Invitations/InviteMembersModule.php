<?php

namespace Wikidot\Modules\Wiki\Invitations;


use Ozone\Framework\SmartyModule;

class InviteMembersModule extends SmartyModule
{

    public function build($runData)
    {

        // check if logged in
        $user = $runData->getUser();
        if (!$user) {
            header("HTTP/1.1 301 Moved Permanently");
            header("Location: " . route('login'));
            return;
        }

        $site = $runData->getTemp("site");
        $runData->contextAdd("site", $site);
        $runData->contextAdd("settings", $site->getSettings());

        $runData->contextAdd("user", $user);
    }
}
