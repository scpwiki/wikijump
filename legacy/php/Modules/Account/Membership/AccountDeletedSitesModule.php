<?php

namespace Wikidot\Modules\Account\Membership;




use Ozone\Framework\Database\Criteria;
use Ozone\Framework\JSONService;
use Wikidot\DB\AdminPeer;
use Wikidot\Utils\AccountBaseModule;

class AccountDeletedSitesModule extends AccountBaseModule
{

    public function build($runData)
    {

        $userId = $runData->getUserId();

        // get all membership - criteria with join - wooo!
        $c = new Criteria();
        $c->add("user_id", $userId);
        $c->addJoin("site_id", "site.site_id");
        $c->add("site.deleted", true);

        $mems = AdminPeer::instance()->select($c);
        if (count($mems)>0) {
            $runData->contextAdd("admins", $mems);
        }

        // get the sites
        $sites = array();
        foreach ($mems as $m) {
            $s = $m->getSite();
            $sites[$s->getSiteId()] = $s->getFieldValuesArray();
            // original unix name...
            $un = $s->getUnixName();
            $un = explode('..del..', $un);
            $un = $un[0];
            $sites[$s->getSiteId()]['slug'] = $un;
        }

        $json = new JSONService(SERVICES_JSON_LOOSE_TYPE);

        $runData->contextAdd('sitesData', $json->encode($sites));
    }
}
