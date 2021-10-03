<?php

namespace Wikidot\DB;

use Illuminate\Support\Facades\DB;
use Ozone\Framework\Database\Criteria;
use Wikidot\Utils\WDStringUtils;

/**
 * Object Model Class.
 *
 */
class PagePeer extends PagePeerBase
{

    public function selectByName($siteId, $name)
    {
        $c = new Criteria();
        $c->add("site_id", $siteId);
        $c->add("unix_name", WDStringUtils::toUnixName($name));
        return $this->selectOne($c);
    }

    public static function getTags($pageId) {
        return json_decode(DB::table('page')->where('page_id', $pageId)->value('tags'));
    }

    public static function saveTags($pageId, $newTags) {
        DB::table('page')
          ->where('page_id', $pageId)
          ->update(['tags' => $newTags]);
    }
}
