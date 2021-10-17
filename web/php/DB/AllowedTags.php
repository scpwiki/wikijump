<?php

namespace Wikidot\DB;

use Ds\Set;
use App\Http\Controllers\Controller;
use Illuminate\Support\Facades\DB;


/**
 * Object Model Class.
 *
 */
class AllowedTags
{

    public static function getEnableTagEngine($site_id): bool {
        return DB::table('site')->where('site_id', $site_id)->value('enable_tag_engine');
    }

    public static function setEnableTagEngine($site_id, $engine_enabled) {
        DB::table('site')
        ->where('site_id', $site_id)
        ->update(['enable_tag_engine' => $engine_enabled]);
    }

    public static function getAllowedTags($site_id): array {
        return json_decode(DB::table('tag_settings')->where('site_id', $site_id)->pluck('allowed_tags')->toArray());
    }

    public static function saveAllowedTags($site_id, $new_tags) {
        // Converts the set to an array, then ensures all tags are unique, sorts the values, and removes any keys, before reconverting to a Set.
        if (!$new_tags->isEmpty()) {
            $new_tags = $new_tags->toArray();

            $new_tags = array_unique($new_tags);
            natsort($new_tags);
            $new_tags = array_values($new_tags);

            $new_tags = new Set($new_tags);
        }

        // Encode to JSON, then update the tags.
        $new_tags = json_encode($new_tags);

        DB::table('tag_settings')
          ->where('site_id', $page_id)
          ->update(['allowed_tags' => $new_tags]);
    }

}
