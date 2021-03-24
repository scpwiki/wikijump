<?php

namespace Wikidot\Modules\ManageSite;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\ThemePeer;
use Wikidot\Utils\ManageSiteBaseModule;

class ManageSiteCustomThemesModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        $site = $runData->getTemp("site");
        $runData->contextAdd("site", $site);

        // now select themes
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("custom", true);
        $c->add("abstract", false);
        $c->addOrderAscending("name");
        $themes = ThemePeer::instance()->select($c);
        $runData->contextAdd("themes", $themes);
    }
}
