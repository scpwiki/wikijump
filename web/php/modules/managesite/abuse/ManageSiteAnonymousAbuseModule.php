<?php
class ManageSiteAnonymousAbuseModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        // get
        $q = "SELECT address, proxy, count(*) AS rank " .
                "FROM anonymous_abuse_flag " .
                "WHERE site_id='".$site->getSiteId()."' " .
                "AND site_valid = TRUE GROUP BY address, proxy ORDER BY rank DESC, address";

        $db = Database::connection();
        $res = $db->query($q);

        $all = $res->fetchAll();

        $runData->contextAdd("reps", $all);
    }
}
