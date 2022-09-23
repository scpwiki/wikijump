<?php

namespace Wikidot\Modules\XList;

use Ozone\Framework\SmartyModule;
use Wikijump\Services\Deepwell\Models\Category;

class WikiCategoriesModule extends SmartyModule
{

    public function build($runData)
    {
        // get categories for the site
        $siteId = $runData->getTemp("site")->getSiteId();
        $categories = Category::findAll($siteId);
        $runData->contextAdd("categories", $categories);
    }
}
