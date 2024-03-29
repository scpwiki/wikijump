<?php

namespace Wikidot\Modules\ManageSite;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\ThemePeer;
use Wikidot\Utils\ManageSiteBaseModule;
use Wikijump\Services\Deepwell\Models\Category;

class ManageSiteAppearanceModule extends ManageSiteBaseModule
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
            $arr = $category->getFieldValuesArray();
            // change themes to conform to variants structure
            if ($arr['theme_id']) {
                $theme = ThemePeer::instance()->selectByPrimaryKey($category->getThemeId());
                if ($theme->getVariantOfThemeId() != null) {
                    $arr['theme_id'] = $theme->getVariantOfThemeId();
                    $arr['variant_theme_id'] = $theme->getThemeId();
                }
            }
            $cats2[] = $arr;
        }
        $runData->ajaxResponseAdd("categories", $cats2);

        // now select themes
        $c = new Criteria();
        $q = "SELECT * from theme WHERE " .
                "abstract = FALSE AND variant_of_theme_id IS NULL " .
                "AND (custom = FALSE" .
                    " OR (custom = TRUE AND site_id='".$site->getSiteId()."')" .
                ") " .
                "ORDER BY custom, sort_index, replace(name, '_', '00000000');";

        $c->setExplicitQuery($q);
        $themes = ThemePeer::instance()->select($c);
        $runData->contextAdd("themes", $themes);

        // get theme variants too
        $c = new Criteria();
        $q = "SELECT * FROM theme WHERE variant_of_theme_id IS NOT NULL ORDER BY name";
        $c->setExplicitQuery($q);
        $variants =  ThemePeer::instance()->select($c);

        $variantsArray = array();
        foreach ($variants as $v) {
            $variantsArray[$v->getVariantOfThemeId()][] = $v;
        }

        $runData->contextAdd("variantsArray", $variantsArray);
    }
}
