<?php

namespace Wikidot\DB;


use Illuminate\Support\Facades\Cache;

/**
 * Object Model Class.
 *
 */
class SiteSettings extends SiteSettingsBase
{

    public function save()
    {
        $key = "sitesettings..".$this->getSiteId();
        Cache::forget($key);
        parent::save();
    }
}
