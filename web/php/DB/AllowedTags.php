<?php

namespace Wikidot\DB;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\AllowedTagsPeer;

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

}
