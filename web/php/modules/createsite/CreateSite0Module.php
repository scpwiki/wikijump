<?php
class CreateSite0Module extends SmartyModule
{

    public function build($runData)
    {

        // can create new wiki now!!!
        $ft = $runData->formTool();
        $form = $ft->getForm("new_site");
        $runData->contextAdd("form", $form);
    }
}
