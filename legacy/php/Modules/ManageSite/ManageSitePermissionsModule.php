<?php

namespace Wikidot\Modules\ManageSite;

use Wikidot\Utils\ManageSiteBaseModule;
use Wikijump\Services\Deepwell\Models\Category;

class ManageSitePermissionsModule extends ManageSiteBaseModule
{

    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $runData->contextAdd("site", $site);

        // get all categories for the site
        $categories = Category::findAll($site->getSiteId());
        $runData->contextAdd("categories", $categories);

        // also prepare categories to put into javascript...
        $cats2 = array();
        foreach ($categories as $category) {
            $cats2[] = $category->getFieldValuesArray();
        }
        $runData->ajaxResponseAdd("categories", $cats2);
    }
}
