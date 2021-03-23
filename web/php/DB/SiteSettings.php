<?php

namespace Wikidot\DB;


use Ozone\Framework\Ozone;

/**
 * Object Model Class.
 *
 */
class SiteSettings extends SiteSettingsBase
{

    public function save()
    {
        $key = "sitesettings..".$this->getSiteId();
        $mc = Ozone::$memcache;
        $s = $mc->delete($key);
        parent::save();
    }
}
