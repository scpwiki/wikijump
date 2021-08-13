<?php

namespace Wikidot\DB;


use Illuminate\Support\Facades\Cache;

/**
 * Object Model Class.
 *
 */
class SiteSuperSettings extends SiteSuperSettingsBase
{

    public function save()
    {
        $key = "sitesupersettings..".$this->getSiteId();
        Cache::forget($key);
        parent::save();
    }
}
