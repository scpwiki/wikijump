<?php

namespace Wikidot\Modules\Wiki\SitesActivity;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Ozone\Framework\Ozone;
use Wikidot\DB\SitePeer;

use Ozone\Framework\SmartyModule;

class MostActiveSitesModule extends SmartyModule
{

    protected $timeOut=3600;

    public function render($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

        $range = $pl->getParameterValue("range", "AMODULE");

        $key = "module..0..MostActiveSitesModule..".$site->getSiteId().'..'.$range;

        $out = Cache::get($key);
        if (!$out) {
            $out = parent::render($runData);
            Cache::put($key, $out, 3600);
        }

        return $out;
    }

    public function build($runData)
    {

        $pl = $runData->getParameterList();

        $range = $pl->getParameterValue("range", "AMODULE");
        $dateStart = new ODate();

        if (!in_array($range, array('24h', '7days', 'month'))) {
            $range = '7days';
        }

        switch ($range) {
            case '24h':
                $dateStart->addSeconds(-60*60*24);
                break;
            case '7days':
                $dateStart->addSeconds(-60*60*24*7);
                break;
            case 'month':
                $dateStart->addSeconds(-60*60*24*31);
                break;
        }
        $q = "SELECT site.site_id, count(*) AS number_changes FROM site, page_revision WHERE page_revision.date_last_edited > '".$dateStart->getDate()."' AND site.visible = TRUE AND site.private = FALSE AND site.deleted = FALSE AND site.site_id != 1 AND page_revision.site_id = site.site_id GROUP BY site.site_id ORDER BY number_changes DESC LIMIT 10";

        $db = Database::connection();

        $res = $db->query($q);

        $all = $res->fetchAll();
        if ($all) {
            foreach ($all as &$a) {
                $a['site'] = SitePeer::instance()->selectByPrimaryKey($a['site_id']);
            }
        }

        $runData->contextAdd("res", $all);
        $runData->contextAdd("range", $range);
    }
}
