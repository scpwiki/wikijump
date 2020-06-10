<?php

// Module CSS
// bluesoul - 2020-06-06
// Licensed Yeezyware
// If you can do it better than me, then you do it.

class CSSModule extends SmartyModule {
    protected $processPage = true;
    public $stylesheet = "";

    public function build($runData)
    {
    }

    public function render($runData)
    {
    $pl = $runData->getParameterList();
        $this->stylesheet = $pl->getParameterValue("module_body");

    //    die(var_dump($this, $runData));
    }

    public function processPage($out, $runData)
    {
        $out = $out . "<style>".$this->stylesheet."</style>";    
        //die(var_dump($this));
        return $out;
    }
}
