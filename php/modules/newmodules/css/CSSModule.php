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
        $pl = $runData->getParameterList();
        $this->stylesheet = $pl->getParameterValue("module_body");

        $csstidy = new csstidy();
        $csstidy->set_cfg('preserve_css', true);
        $csstidy->parse($this->stylesheet);
        $csstidy->print->formatted();

        $this->stylesheet = $csstidy->print->output_css_plain;

        $runData->contextAdd('stylesheet', $this->stylesheet);
    }
}
