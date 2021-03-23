<?php

namespace Wikidot\DB;


use Ozone\Framework\Ozone;

/**
 * Object Model Class.
 *
 */
class SiteSuperSettings extends SiteSuperSettingsBase
{

    public function save()
    {
        $key = "sitesupersettings..".$this->getSiteId();
        $mc = Ozone::$memcache;
        $s = $mc->delete($key);
        parent::save();
    }
}
