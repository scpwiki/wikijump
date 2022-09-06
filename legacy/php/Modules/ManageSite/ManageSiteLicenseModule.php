<?php

namespace Wikidot\Modules\ManageSite;

use Wikijump\Services\License\LicenseMapping;
use Wikijump\Services\Deepwell\Models\Category;

class ManageSiteLicenseModule extends ManageSiteBaseModule
{
    public function build($runData)
    {
        $site = $runData->getTemp('site');
        $runData->contextAdd('site', $site);

        // get all categories for the site
        $categories = Category::findAll($site->getSiteId());
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
