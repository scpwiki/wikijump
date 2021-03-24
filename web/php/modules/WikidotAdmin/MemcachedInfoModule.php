<?php

namespace Wikidot\Modules\WikidotAdmin;

use Ozone\Framework\Ozone;
use Ozone\Framework\SmartyModule;

class MemcachedInfoModule extends SmartyModule
{

    public function build($runData)
    {
        $mc = Ozone::$memcache;
        $raw = $mc->getExtendedStats();
        $runData->contextAdd("raw", $raw);
    }
}
