<?php

namespace Wikidot\Modules\CreateSite;


use Ozone\Framework\SmartyModule;

class NotLoggedModule extends SmartyModule
{

    public function build($runData)
    {

        $ft = $runData->formTool();
        $form = $ft->getForm("login_user");
        $runData->contextAdd("login_form", $form);
    }
}
