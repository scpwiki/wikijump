<?php

namespace Wikidot\Modules\Wiki\HotTags;

use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\SmartyModule;
use Wikijump\Services\Deepwell\Models\Category;

class PagesListByTagModule extends SmartyModule
{

    public function render($runData)
    {
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $threadId = $pl->getParameterValue("t");

        $parmHash = md5(serialize($pl->asArray()));

        $key = 'list_pages_by_tags_v..'.$site->getSiteId().'..'.$parmHash;
        $tkey = 'page_tags_lc..'.$site->getSiteId(); // last change timestamp

        $struct = Cache::get($key);

        $cacheTimestamp = $struct['timestamp'];
        $changeTimestamp = Cache::get($tkey);

        if ($struct) {
            // check the times

            if ($changeTimestamp && $changeTimestamp <= $cacheTimestamp) {
                $out = $struct['content'];
                return $out;
            }
        }

        $out = parent::render($runData);

        // and store the data now
        $struct = array();
        $now = time();
        $struct['timestamp'] = $now;
        $struct['content'] = $out;

        Cache::put($key, $struct, 1000);

        if (!$changeTimestamp) {
            $changeTimestamp = $now;
            Cache::put($tkey, $changeTimestamp, 3600);
        }

        return $out;
    }

    public function build($runData)
    {
        $pl = $runData->getParameterList();

        $site = $runData->getTemp("site");

        $tag = $pl->getParameterValue("tag");
        if ($tag === null) {
            $runData->setModuleTemplate("Empty");
            return '';
        }

        // get pages

        $categoryName = $pl->getParameterValue("category");
        $category = null;
        if ($categoryName) {
            $category = Category::findSlug($site->getSiteId(), $categoryName);
            if ($category === null) {
                return '';
            }
            $runData->contextAdd("category", $category);
        }

        $c = new Criteria();
        $c->setExplicitFrom("page");
        $c->add("tags", $tag);
        $c->add("site_id", $site->getSiteId());
        if ($category) {
            $c->add("category_id", $category->getCategoryId());
        }
        $c->addOrderAscending('COALESCE(title, unix_name)');

        $pages = [null]; // TODO run query

    //  $q = "SELECT site.* FROM site, tag WHERE tag.tag = '".db_escape_string($tag")."'

        $runData->contextAdd("tag", $tag);
        $runData->contextAdd("pages", $pages);
        $runData->contextAdd("pageCount", count($pages));

        $runData->contextAdd("pageUnixName", $runData->getTemp("page")->getUnixName());
    }
}
