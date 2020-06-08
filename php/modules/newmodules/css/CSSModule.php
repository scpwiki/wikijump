<?php

// Module CSS
// bluesoul - 2020-06-06
// Licensed Yeezyware
// If you can do it better than me, then you do it.

class CSSModule extends SmartyModule {
    protected $processPage = true;

    public function build($runData)
    {

    }

    public function render($runData)
    {

    }

    public function processPage($out, $runData)
    {
        $out = preg_replace_callback("/\[\[module CSS\s?.*\]\](.*)\[\[\/module\]\]/", function($match) {
            return "<style>".$match[0]."</style>";
        }, $out);
        return $out;
    }
}