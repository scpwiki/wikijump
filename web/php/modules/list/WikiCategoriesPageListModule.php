<?php
use DB\PagePeer;

class WikiCategoriesPageListModule extends SmartyModule
{

    public function build($runData)
    {
        $categoryId = $runData->getParameterList()->getParameterValue("category_id");

        $site = $runData->getTemp("site");

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("category_id", $categoryId);
        $c->addOrderAscending("COALESCE(title, unix_name)");
        $pages = PagePeer::instance()->select($c);

        if (count($pages)>0) {
            $runData->contextAdd("pages", $pages);
        }

        $runData->ajaxResponseAdd("categoryId", $categoryId);
    }
}
