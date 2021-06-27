<?php

namespace Wikidot\DB;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\SiteTagPeer;

/**
 * Object Model Class.
 *
 */
class SiteTag extends SiteTagBase
{
    public static function getSiteTags($siteId): string {
        $c = new Criteria();
        $c->add("site_id", $siteId);
        $dbTags = SiteTagPeer::instance()->selectByCriteria($c);
        $siteTags = '';
        foreach ($dbTags as $dbTag) {
            $siteTags .= htmlspecialchars($dbTag->getTag()).' ';
        }
        return trim($siteTags);
    }

}
