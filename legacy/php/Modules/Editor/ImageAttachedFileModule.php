<?php

namespace Wikidot\Modules\Editor;


use Ozone\Framework\Database\Criteria;
use Ozone\Framework\SmartyModule;
use Wikijump\Services\Deepwell\Models\File;

class ImageAttachedFileModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();

        $pageId = $pl->getParameterValue("pageId");
        $files = File::findFromPage($pageId);
        $runData->contextAdd("files", $files);
    }
}
