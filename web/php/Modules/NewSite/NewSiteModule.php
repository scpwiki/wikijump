<?php

namespace Wikidot\Modules\NewSite;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\SitePeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\WDStringUtils;

class NewSiteModule extends SmartyModule
{

    public function build($runData)
    {
        if ($runData->getUser() == null) {
            $runData->contextAdd("notLogged", true);
        }

        $pl = $runData->getParameterList();
        $siteUnixName = WDStringUtils::toUnixName($pl->getParameterValue('address') ?? '');
        $runData->contextAdd('unixName', $siteUnixName);

        $siteName = str_replace('-', ' ', $siteUnixName);
        $siteName = ucwords($siteName);
        $runData->contextAdd('siteName', $siteName);

        // get template sites
        $c = new Criteria();
        $c->add('unix_name', '^template-', '~');
        $c->addOrderAscending('site_id');
        $templates = SitePeer::instance()->select($c);
        $runData->contextAdd('templates', $templates);
    }
}
