<?php

namespace Wikidot\Modules\PageRate;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\GlobalProperties;
use Wikijump\Services\Deepwell\Models\Page;

class PageRateModule extends SmartyModule
{

    public function build($runData)
    {

        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("pageId");

        $page = Page::findIdOnly($pageId);
        // todo: check if allowed

        $runData->contextAdd("pageId", $page->getPageId());

        $uri = GlobalProperties::$MODULES_CSS_URL.'/PageRate/PageRateWidgetModule.css';
        $this->extraCss[] = $uri;

        $uri = GlobalProperties::$MODULES_JS_URL.'/PageRate/PageRateWidgetModule.js';
        $this->extraJs[] = $uri;

        //check if voters visible
        $category = $page->getCategory();
        $runData->contextAdd("visibility", $category->getRatingVisible());
    }
}
