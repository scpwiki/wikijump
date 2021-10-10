<?php

namespace Wikidot\Modules\ManageSite;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\CategoryPeer;
use Wikidot\Utils\ManageSiteBaseModule;
use Wikijump\Services\License\LicenseMapping;

class ManageSiteLicenseModule extends ManageSiteBaseModule
{
    public function build($runData)
    {
        $site = $runData->getTemp('site');
        $runData->contextAdd('site', $site);

        // get all categories for the site
        $c = new Criteria();
        $c->add('site_id', $site->getSiteId());
        $c->addOrderAscending("replace(name, '_', '00000000')"); // Weird wikidot hack to make "_default" et all appear at the top
        $categories = CategoryPeer::instance()->select($c);

        $runData->contextAdd('categories', $categories);

        // also prepare categories to put into javascript...
        $cats2 = [];
        foreach ($categories as $category) {
            array_push($cats2, $category->getFieldValuesArray());
        }
        $runData->ajaxResponseAdd('categories', $cats2);

        // Add license data
        $runData->contextAdd('licenses', LicenseMapping::list());
    }
}
