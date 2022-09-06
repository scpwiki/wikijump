<?php

namespace Wikidot\Modules\CreateSite;


use Ozone\Framework\SmartyModule;

class CreateSite0Module extends SmartyModule
{

    public function build($runData)
    {

        // can create new Wiki now!!!
        $ft = $runData->formTool();
        $form = $ft->getForm("new_site");
        $runData->contextAdd("form", $form);
    }
}
