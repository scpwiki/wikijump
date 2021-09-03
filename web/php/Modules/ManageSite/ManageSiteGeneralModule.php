<?php

namespace Wikidot\Modules\ManageSite;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\AllowedTagsPeer;
use Wikidot\DB\AllowedTags;
use Wikidot\Utils\ManageSiteBaseModule;


class ManageSiteGeneralModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        $site = $runData->getTemp("site");
        $siteId = $site->getSiteId();

        $allowedTags = AllowedTags::getAllowedTags($siteId);
        $enableAllowedTags = AllowedTags::getEnableAllowedTags($siteId);

        $runData->contextAdd("allowedTags", $allowedTags);
        $runData->contextAdd("enableAllowedTags", $enableAllowedTags);
        $runData->contextAdd("site", $site);
    }
}
