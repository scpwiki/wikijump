<?php

namespace Wikidot\DB;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Wikidot\Utils\GlobalProperties;

/**
 * Object Model Class.
 *
 */
class Site extends SiteBase
{
    public function getDomain()
    {
        if ($this->getCustomDomain() == null || $this->getCustomDomain() == "") {
            return $this->getUnixName() . "." . GlobalProperties::$URL_DOMAIN;
        } else {
            return $this->getCustomDomain();
        }
    }

    public function getSettings()
    {
        $key = "sitesettings.." . $this->getSiteId();
        $s = Cache::get($key);
        if (!$s) {
            $c = new Criteria();
            $c->add("site_id", $this->getSiteId());
            $s = SiteSettingsPeer::instance()->selectOne($c);
            Cache::put($key, $s, 864000);
        }
        return $s;
    }

    public function getForumSettings()
    {
        $c = new Criteria();
        $c->add("site_id", $this->getSiteId());
        return ForumSettingsPeer::instance()->selectOne($c);
    }

    public function save()
    {
        $key = 'site..' . $this->getUnixName();
        Cache::forget($key);
        $key = 'site_cd..' . $this->getCustomDomain();
        Cache::forget($key);
        parent::save();
    }

    public function getLocalFilesPath()
    {
        return WIKIJUMP_ROOT . '/web/files--sites/'.$this->getUnixName();

        /* optional hashing */
        $un = $this->getUnixName();
        $p = substr($un, 0, 1) . '/' . substr($un, 0, 2) . '/' . $un;

        return WIKIJUMP_ROOT . '/web/files--sites/' . $p;
    }
}
