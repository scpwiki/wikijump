<?php

namespace Wikidot\Modules\ManageSite;

use Ozone\Framework\Database\Criteria;

use Wikidot\Utils\ManageSiteBaseModule;

class ManageSitePrivateSettingsModule extends ManageSiteBaseModule
{

    protected $processPage = true;

    public function build($runData)
    {

        $site = $runData->getTemp("site");
        $runData->contextAdd("site", $site);
        $runData->contextAdd("settings", $site->getSettings());
        $runData->contextAdd("superSettings", $site->getSuperSettings());

        $runData->contextAdd("viewers", null);
        $runData->contextAdd("settings", $site->getSettings());
    }
}
