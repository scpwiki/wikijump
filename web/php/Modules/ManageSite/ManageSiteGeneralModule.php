<?php

namespace Wikidot\Modules\ManageSite;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\SiteTagPeer;
use Wikidot\DB\SiteTag;
use Wikidot\Utils\ManageSiteBaseModule;

class ManageSiteGeneralModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        $site = $runData->getTemp("site");
        $siteId = $site->getSiteId();

        $defaultTags = SiteTag::getSiteTags($siteId);

        $runData->contextAdd("defaultTags", $defaultTags);
        $runData->contextAdd("site", $site);
    }
}
