<?php

namespace Wikidot\Modules\PageRate;

use Wikidot\DB\PagePeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\GlobalProperties;

class PageRateModule extends SmartyModule
{

    public function build($runData)
    {

        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("pageId");

        $page = PagePeer::instance()->selectByPrimaryKey($pageId);
        // todo: check if allowed

        $runData->contextAdd("pageId", $page->getPageId());

        $uri = GlobalProperties::$MODULES_CSS_URL.'/pagerate/PageRateWidgetModule.css';
        $this->extraCss[] = $uri;

        $uri = GlobalProperties::$MODULES_JS_URL.'/pagerate/PageRateWidgetModule.js';
        $this->extraJs[] = $uri;

        //check if voters visible
        $category = $page->getCategory();
        $runData->contextAdd("visibility", $category->getRatingVisible());
    }
}
