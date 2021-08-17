<?php

namespace Wikidot\Utils;

use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Ozone;
use Ozone\Framework\SmartyModule;

abstract class CacheableModule extends SmartyModule
{

    protected $timeOut = 0;
    protected $minTimeOut = 0; // important when $allowChangeTimeOut == true
    protected $maxTimeOut = 86400; // --||--

    /**
     * Whether timeout can be changed e.g. by passing timeout="1212" parameter in Wiki source.
     */
    protected $allowChangeTimeOut = false;

    /**
     * Overrides original method and adds caching mechanisms.
     */
    public function render($runData)
    {

        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");
        // first determine: to cache or not to cache:
        $uTimeOut = $pl->getParameterValue("timeout");
        if ($this->allowChangeTimeOut == true && $uTimeOut != null && $uTimeOut >0) {
            $timeOut = $uTimeOut;
            // confront with max and min
            if ($timeOut > $this->maxTimeOut) {
                $timeOut = $this->maxTimeOut;
            }
            if ($timeOut < $this->minTimeOut) {
                $timeOut = $this->minTimeOut;
            }
        } else {
            $timeOut = $this->timeOut;
            // do not check max and min - we should trust this value and do not complicate things
        }

        if ($timeOut != null && $timeOut > 0) {
            // cacheable
            $parmSubKey = md5(serialize($pl->asArray()));

            $mcKey = 'module..'.$site->getSiteId().'..'.get_class($this).'..'.$parmSubKey;

            // get the content
            $out = Cache::get($mcKey);
            if ($out != false) {
                return $out;
            }

            $storeLater = true;

            $out = parent::render($runData);

            if ($storeLater) {
                Cache::put($mcKey,$out,$timeOut);
            }

            return $out;
        } else {
            return  parent::render($runData);
        }
    }
}
