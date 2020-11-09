<?php
class NewWikiModule extends SmartyModule
{

    public function build($runData)
    {

        if (!$runData->isUserAuthenticated()) {
            $ft = $runData->formTool();
            $form = $ft->getForm("login_user");
            $runData->contextAdd("login_form", $form);
            $runData->setModuleTemplate("NewWiki/NotLoggedModule");
        } else {
            // can create new wiki now!!!
            $ft = $runData->formTool();
            $form = $ft->getForm("new_site");
            $runData->contextAdd("form", $form);
        }
    }
}
