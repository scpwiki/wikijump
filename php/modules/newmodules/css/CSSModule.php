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

        // Thanks dleavitt@StackOverflow.
        // https://stackoverflow.com/questions/3241616/sanitize-user-defined-css-in-php/5209050#5209050
        // Instantiate Purifier config and instance.
        $config = HTMLPurifier_Config::createDefault();
        $config->set('Filter.ExtractStyleBlocks', TRUE);
        $purifier = new HTMLPurifier($config);

        // Turn off strict warnings (CSSTidy throws some warnings on PHP 5.2+)
        $level = error_reporting(E_ALL & ~E_STRICT);

        // Wrap our CSS in style tags and pass to purifier.
        // we're not actually interested in the html response though
        $html = $purifier->purify('<style>'.$this->stylesheet.'</style>');

        // Revert error reporting
        error_reporting($level);

        // The "style" blocks are stored seperately
        $output_css = $purifier->context->get('StyleBlocks');

        // Implode all style blocks to a string.
        $this->stylesheet = implode('',$output_css);

        $runData->contextAdd('stylesheet', $this->stylesheet);
    }
}
