<?php
class MemcachedInfoModule extends SmartyModule
{

    public function build($runData)
    {
        $mc = Ozone::$memcache;
        $raw = $mc->getExtendedStats();
        $runData->contextAdd("raw", $raw);
    }
}
