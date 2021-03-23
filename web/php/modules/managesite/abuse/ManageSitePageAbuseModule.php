<?php

namespace Wikidot\Modules\ManageSite\Abuse;

use Ozone\Framework\Database\Database;
use Wikidot\Utils\ManageSiteBaseModule;

class ManageSitePageAbuseModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        // get

        $q = "SELECT path, count(*) AS rank " .
                "FROM page_abuse_flag " .
                "WHERE site_id='".$site->getSiteId()."' " .
                "AND site_valid = TRUE GROUP BY path " .
                "ORDER BY rank DESC, path";

        $db = Database::connection();
        $res = $db->query($q);

        $all = $res->fetchAll();

        $runData->contextAdd("reps", $all);
    }
}
