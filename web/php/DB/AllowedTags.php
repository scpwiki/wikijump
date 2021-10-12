<?php

namespace Wikidot\DB;

use Wikidot\DB\AllowedTagsPeer;
use App\Http\Controllers\Controller;
use Illuminate\Support\Facades\DB;


/**
 * Object Model Class.
 *
 */
class AllowedTags extends AllowedTagsBase
{
    public static function getAllowedTags($siteId): array {
        return DB::table('tag_settings')->where('site_id', $siteId)->pluck('allowed_tags')->toArray();

    }

    public static function getEnableTagEngine($siteId): bool {
        return DB::table('site')->where('site_id', $siteId)->value('enable_tag_engine');
    }

}
