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
    public static function getAllowedTags($siteId): string {
        $c = new Criteria();
        $c->add("site_id", $siteId);
        $dbTags = AllowedTagsPeer::instance()->selectByCriteria($c);
        $allowedTags = '';
        foreach ($dbTags as $dbTag) {
            $allowedTags .= htmlspecialchars($dbTag->getTag()).' ';
        }
        return trim($allowedTags);
    }

    public static function getEnableAllowedTags($siteId): bool {
        $enableAllowedTags = DB::table('site')->where('site_id', $siteId)->value('enable_allowed_tags');
        return $enableAllowedTags;
    }

}
