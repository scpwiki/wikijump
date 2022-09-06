<?php

namespace Wikidot\Modules\Wiki\SitesActivity;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Database;
use Ozone\Framework\Ozone;
use Ozone\Framework\SmartyModule;

class SomeGlobalStatsModule extends SmartyModule
{

    protected $timeOut=3600;

    public function render($runData)
    {
        $pl = $runData->getParameterList();

        $site = $runData->getTemp("site");
        $range = $pl->getParameterValue("range", "AMODULE");

        $key = "module..0..SomeGlobalStatsModule";

        $out = Cache::get($key);
        if (!$out) {
            $out = parent::render($runData);
            Cache::put($key, $out, 600);
        }

        return $out;
    }

    public function build($runData)
    {
        // just get some numbers

        $db = Database::connection();

        $q = "SELECT count(*) AS c FROM ozone_user";
        $res = $db->query($q);
        $row = $res->nextRow();
        $totalUsers = $row['c'];
        /* -2 because there are 2 "ghost users" in the default installation
            with user_id < 0 */
        $runData->contextAdd("totalUsers", $totalUsers - 2);

        $q = "SELECT count(*) AS c FROM site";
        $res = $db->query($q);
        $row = $res->nextRow();
        $totalSites = $row['c'];
        $runData->contextAdd("totalSites", $totalSites);

        $q = "SELECT count(*) AS c FROM page";
        $res = $db->query($q);
        $row = $res->nextRow();
        $totalPages = $row['c'];
        $runData->contextAdd("totalPages", $totalPages);

        $q = "SELECT count(*) AS c FROM ozone_user WHERE registered_date>now() - interval '1 day'";
        $res = $db->query($q);
        $row = $res->nextRow();
        $newUsers = $row['c'];
        $runData->contextAdd("newUsers", $newUsers);

        $q = "SELECT count(*) AS c FROM page_revision WHERE date_last_edited>now() - interval '1 day'";
        $res = $db->query($q);
        $row = $res->nextRow();
        $recentEdits = $row['c'];
        $runData->contextAdd("recentEdits", $recentEdits);
    }
}
