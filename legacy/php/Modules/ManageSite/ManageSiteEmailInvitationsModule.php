<?php

namespace Wikidot\Modules\ManageSite;

use Wikidot\Utils\ManageSiteBaseModule;

class ManageSiteEmailInvitationsModule extends ManageSiteBaseModule
{

    protected $processPage = true;

    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $runData->contextAdd("site", $site);
        $runData->contextAdd("settings", $site->getSettings());

        $runData->contextAdd("user", $runData->getUser());
    }
}
