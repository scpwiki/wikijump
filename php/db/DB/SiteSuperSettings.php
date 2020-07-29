<?php
namespace DB;

/**
 * Object Model class.
 *
 */
class SiteSuperSettings extends SiteSuperSettingsBase
{

    public function save()
    {
        $key = "sitesupersettings..".$this->getSiteId();
        $mc = \Ozone::$memcache;
        $s = $mc->delete($key);
        parent::save();
    }
}
