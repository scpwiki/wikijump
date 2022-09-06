<?php

namespace Wikidot\Modules\Edit;

use Ozone\Framework\SmartyModule;
use Wikijump\Services\Deepwell\Models\Page;

class TemplateSourceModule extends SmartyModule
{

    public function build($runData)
    {
        $pageId = $runData->getParameterList()->getParameterValue("page_id");
        $page = Page::findIdOnly($pageId);
        $source = $page->getSource();
        $runData->contextAdd("source", $source);
    }
}
