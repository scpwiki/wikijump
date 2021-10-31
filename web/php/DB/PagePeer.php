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

    public function selectByName(string $site_id, string $name)
    {
        $c = new Criteria();
        $c->add("site_id", $site_id);
        $c->add("unix_name", WDStringUtils::toUnixName($name));
        return $this->selectOne($c);
    }

    public static function getTags(string $page_id): Set {
        $fetched_tags = DB::table('page')
            ->where('page_id', $page_id)
            ->value('tags');

        $fetched_tags = json_decode($fetched_tags); // Decodes the tags.
        return $fetched_tags === null ? new Set() : new Set($fetched_tags); // Convert to set, and if null, return empty set.
    }

    public static function saveTags(stringn $page_id, Set $tags) {
        // Converts the set to an array, then ensures all tags are unique, sorts the values, and removes any keys.
        $tag_array = $tags->toArray();
        natsort($tag_array);
        $tag_array = array_values($tag_array);

        // Update the tags.
        DB::table('page')
          ->where('page_id', $page_id)
          ->update(['tags' => $tag_array]);
    }
}
