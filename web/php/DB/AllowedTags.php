<?php

namespace Wikidot\DB;

use App\Http\Controllers\Controller;
use Illuminate\Support\Facades\DB;


/**
 * Object Model Class.
 *
 */
class AllowedTags
{

    public static function getEnableTagEngine($siteId): bool {
        return DB::table('site')->where('site_id', $siteId)->value('enable_tag_engine');
    }

    public static function setEnableTagEngine($siteId, $engineEnabled) {
        DB::table('site')
        ->where('site_id', $siteId)
        ->update(['enable_tag_engine' => $engineEnabled]);
    }

    public static function getAllowedTags($siteId): array {
        return json_decode(DB::table('tag_settings')->where('site_id', $siteId)->pluck('allowed_tags')->toArray());
    }

    public static function saveAllowedTags($siteId, $newTags) {
        if ($newTags !== '') {
            $newTags = array_unique($newTags);
            natsort($newTags);
            $newTags = array_values($newTags);
        } else {
            $newTags = [];
        }

        // Update the tags.
        DB::table('tag_settings')
          ->where('site_id', $pageId)
          ->update(['allowed_tags' => $newTags]);
    }

}
