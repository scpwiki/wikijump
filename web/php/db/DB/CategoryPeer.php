<?php

namespace DB;

use \WDStringUtils;
use Criteria;





/**
 * Object Model mapped class.
 *
 */
class CategoryPeer extends CategoryPeerBase
{

    public function selectByName($name, $siteId, $useMemcache = true)
    {

        if ($name !== "_default") {
            $name = WDStringUtils::toUnixName($name);
        }

        if ($useMemcache) {
            $memcache = \Ozone::$memcache;
            $key = 'category..'.$siteId.'..'.$name;
            $cat = $memcache->get($key);
            if ($cat) {
                return $cat;
            } else {
                $c = new Criteria();
                $c->add("name", $name);
                $c->add("site_id", $siteId);
                $cat = $this->selectOne($c);
                $memcache->set($key, $cat, 0, 864000); // 10 days ;-)
                return $cat;
            }
        } else {
            $c = new Criteria();
            $c->add("name", $name);
            $c->add("site_id", $siteId);
            $cat = $this->selectOne($c);
            return $cat;
        }
    }

    public function selectById($categoryId, $siteId, $useMemcache = true)
    {

        if ($useMemcache) {
            $memcache = \Ozone::$memcache;
            $key = 'categorybyid..'.$siteId.'..'.$categoryId;
            $cat = $memcache->get($key);
            if ($cat != false) {
                return $cat;
            } else {
                $c = new Criteria();
                $c->add("category_id", $categoryId);
                $c->add("site_id", $siteId);
                $cat = $this->selectOne($c);
                $memcache->set($key, $cat, 0, 864000);
                return $cat;
            }
        } else {
            $c = new Criteria();
            $c->add("category_id", $categoryId);
            $c->add("site_id", $siteId);
            $cat = $this->selectOne($c);
            return $cat;
        }
    }
}
