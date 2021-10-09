<?php

namespace Wikidot\DB;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Ozone;
use Wikidot\Utils\WDStringUtils;
use Ozone\Framework\Database\Criteria;

/**
 * Object Model mapped Class.
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
            $key = 'category..'.$siteId.'..'.$name;
            $cat = Cache::get($key);
            if ($cat) {
                return $cat;
            } else {
                $c = new Criteria();
                $c->add("name", $name);
                $c->add("site_id", $siteId);
                $cat = $this->selectOne($c);
                Cache::put($key, $cat, 864000);
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
            $key = 'categorybyid..'.$siteId.'..'.$categoryId;
            $cat = Cache::get($key);
            if ($cat != false) {
                return $cat;
            } else {
                $c = new Criteria();
                $c->add("category_id", $categoryId);
                $c->add("site_id", $siteId);
                $cat = $this->selectOne($c);
                Cache::put($key, $cat, 864000);
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
