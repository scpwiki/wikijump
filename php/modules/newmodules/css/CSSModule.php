<?php

// Module CSS
// bluesoul - 2020-06-06
// Licensed Yeezyware
// If you can do it better than me, then you do it.

class CSSModule extends SmartyModule
{
    protected $processPage = false;
    public $stylesheet = "";

    public function build($runData)
    {
    }

    public function render($runData)
    {
        $pl = $runData->getParameterList();
        $this->stylesheet = $pl->getParameterValue("module_body");

        if($runData->getModuleTemplate() == null){return;}

        $this->build($runData);

        $template = $runData->getModuleTemplate();
        $templateFile  = PathManager::moduleTemplate($template);
        // render!

        $smarty = Ozone::getSmartyPlain();

        $page = $runData->getPage();
        $smarty->assign("page", $page);

        // put context into context

        $context = $runData->getContext();
        if($context !== null){
            foreach($context as $key => $value){
                $smarty->assign($key, $value);
            }
        }

        // put errorMessages and messages into the smarty's context as well.
        $dataMessages = $runData->getMessages();
        $dataErrorMessages = $runData->getErrorMessages();
        if(count($dataMessages) > 0) {
            $smarty->assign('data_messages', $dataMessages);
        }

        if(count($dataErrorMessages) > 0) {
            $smarty->assign('data_errorMessages', $dataErrorMessages);
        }
        $csstidy = new csstidy();
        $csstidy->set_cfg('preserve_css', true);
        $csstidy->parse($this->stylesheet);
        $csstidy->print->formatted();

        $this->stylesheet = $csstidy->print->output_css_plain;

        $smarty->assign('stylesheet', $this->stylesheet);
        $out = $smarty->fetch($templateFile);

        return $out;

    }
}
