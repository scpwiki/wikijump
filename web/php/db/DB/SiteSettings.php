<?php
namespace DB;

/**
 * Object Model class.
 *
 */
class SiteSettings extends SiteSettingsBase
{

    public function save()
    {
        $key = "sitesettings..".$this->getSiteId();
        $mc = \Ozone::$memcache;
        $s = $mc->delete($key);
        parent::save();
    }
}
