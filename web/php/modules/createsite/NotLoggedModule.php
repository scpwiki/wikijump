<?php
class NotLoggedModule extends SmartyModule
{

    public function build($runData)
    {

        $ft = $runData->formTool();
        $form = $ft->getForm("login_user");
        $runData->contextAdd("login_form", $form);
    }
}
