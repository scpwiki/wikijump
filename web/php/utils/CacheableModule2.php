<?php

namespace Wikidot\Utils;

/**
 * A better way to cache
 */
use Ozone\Framework\SmartyModule;

abstract class CacheableModule2 extends SmartyModule
{

    protected $keyBase;
    protected $timeOut = 3600;
    protected $delay = 0;

    protected $keyFull = null;
    protected $keyFullTimestamp = null;

    public function render($runData)
    {
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();

        $parmArray = $pl->asArray();
        $parmHash = md5(serialize($parmArray).$runData->getModuleTemplate());

        if ($this->keyFull) {
            $key = $this->keyFull;
        } else {
            $key = $this->keyBase.'_v..'.$site->getSiteId().'..'.$parmHash;
        }
        if ($this->keyFullTimestamp) {
            $tkey = $this->keyFullTimestamp;
        } else {
            $tkey = $this->keyBase.'_lc..'.$site->getSiteId(); // last change timestamp
        }

        $mc = OZONE::$memcache;
        $struct = $mc->get($key);

        $cacheTimestamp = $struct['timestamp'];
        $changeTimestamp = $mc->get($tkey);

        if ($struct) {
            // check the times

            if ($changeTimestamp && $changeTimestamp <= $cacheTimestamp + $this->delay) {
                $out = $struct['content'];
                return $out;
            }
        }

        $out = parent::render($runData);

        // and store the data now
        $struct = array();
        $now = time();
        $struct['timestamp'] = $now;
        $struct['content'] = $out;

        $mc->set($key, $struct, 0, $this->timeOut);

        if (!$changeTimestamp) {
            $changeTimestamp = $now;
            $mc->set($tkey, $changeTimestamp, 0, $this->timeOut);
        }

        return $out;
    }

    protected function _compareMicrotime($t1, $t2)
    {
        $t1 = explode(' ', $t1);
        $t2 = explode(' ', $t2);
        if ($t1[1]<$t2[1]) {
            return -1;
        }
        if ($t1[1]>$t2[1]) {
            return 1;
        }
        if ($t1[1] == $t2[1]) {
            if ($t1[0]<$t2[0]) {
                return -1;
            }
            if ($t1[0]>$t2[0]) {
                return 1;
            }
            if ($t1[0]==$t2[0]) {
                return 0;
            }
        }
    }
}
