<?php

namespace Wikidot\Modules\ManageSite;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\ThemePeer;
use Wikidot\Utils\ManageSiteBaseModule;

class ManageSiteNavigationModule extends ManageSiteBaseModule
{

    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $runData->contextAdd("site", $site);

        // get all categories for the site
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->addOrderAscending("replace(name, '_', '00000000')");
        $categories = CategoryPeer::instance()->select($c);

        $runData->contextAdd("categories", $categories);

        // also prepare categories to put into javascript...
        $cats2 = array();
        foreach ($categories as $category) {
            $cats2[] = $category->getFieldValuesArray();
        }
        $runData->ajaxResponseAdd("categories", $cats2);

        // now select themes
        $c = new Criteria();
        $c->addOrderAscending("name");
        $themes = ThemePeer::instance()->select($c);
        $runData->contextAdd("themes", $themes);
    }
}
