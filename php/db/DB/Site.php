<?php
namespace DB;

use Criteria;
use \GlobalProperties;

/**
 * Object Model class.
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
        $mc = \Ozone::$memcache;
        $s = $mc->get($key);
        if (!$s) {
            $c = new Criteria();
            $c->add("site_id", $this->getSiteId());
            $s = SiteSettingsPeer::instance()->selectOne($c);
            $mc->set($key, $s, 0, 864000);
        }
        return $s;
    }

    public function getSuperSettings()
    {

        $s = SiteSuperSettingsPeer::instance()->selectByPrimaryKey($this->getSiteId());

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
        $memcache = \Ozone::$memcache;
        $key = 'site..' . $this->getUnixName();
        $memcache->delete($key);
        $key = 'site_cd..' . $this->getCustomDomain();
        $memcache->delete($key);
        parent::save();
    }

    public function getLocalFilesPath()
    {
        return WIKIDOT_ROOT . '/web/files--sites/'.$this->getUnixName();

        /* optional hashing */
        $un = $this->getUnixName();
        $p = substr($un, 0, 1) . '/' . substr($un, 0, 2) . '/' . $un;

        return WIKIDOT_ROOT . '/web/files--sites/' . $p;
    }
}
