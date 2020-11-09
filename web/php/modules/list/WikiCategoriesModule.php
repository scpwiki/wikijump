<?php
use DB\CategoryPeer;

class WikiCategoriesModule extends SmartyModule
{

    public function build($runData)
    {
        // get categories for the site

        $siteId = $runData->getTemp("site")->getSiteId();

        $c = new Criteria();
        $c->add("site_id", $siteId);
        $c->addOrderAscending("replace(name, '_', '00000000')");

        $cats = CategoryPeer::instance()->select($c);

        $runData->contextAdd("categories", $cats);
    }
}
