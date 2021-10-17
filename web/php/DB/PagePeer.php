<?php

namespace Wikidot\DB;

use Ds\Set;
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

    public static function getTags($pageId): set {
        $fetched_tags = DB::table('page')->where('page_id', $pageId)->value('tags');
        $fetched_tags = json_decode($fetched_tags); // Decodes the tags. 
        return $fetched_tags === null ? new Set() : new Set($fetched_tags); // Convert to set, and if null, return empty set.
    }

    public static function saveTags($page_id, set $new_tags) {
        // Converts the set to an array, then ensures all tags are unique, sorts the values, and removes any keys.
        if (!$new_tags->isEmpty()) {
            $new_tags = $new_tags->toArray();
            $new_tags = array_unique($new_tags);
            natsort($new_tags);
            $new_tags = array_values($new_tags);
        } else {
            $new_tags = [];
        }

        // Update the tags.
        DB::table('page')
          ->where('page_id', $page_id)
          ->update(['tags' => $new_tags]);
    }
}
