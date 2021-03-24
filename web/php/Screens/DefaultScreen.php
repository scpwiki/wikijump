<?php

namespace Wikidot\Screens;

use Ozone\Framework\SmartyScreen;

class DefaultScreen extends SmartyScreen
{

    public function build($runData)
    {
        $runData->contextAdd("testkey", "testval");
    }
}
