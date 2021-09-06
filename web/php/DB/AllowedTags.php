<?php

namespace Wikidot\DB;

use Ozone\Framework\Database\Criteria;
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
        return DB::table('site_tag')->where('site_id', $siteId)->pluck('tag')->toArray();

    }

    public static function getEnableAllowedTags($siteId): bool {
        return DB::table('site')->where('site_id', $siteId)->value('enable_allowed_tags');
    }

}
