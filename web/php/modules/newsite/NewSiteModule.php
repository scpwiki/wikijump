<?php
use DB\SitePeer;

class NewSiteModule extends SmartyModule
{

    public function build($runData)
    {
        if ($runData->getUser() == null) {
            $runData->contextAdd("notLogged", true);
        } else {
//
//
        }
        $pl = $runData->getParameterList();
        $siteUnixName = WDStringUtils::toUnixName($pl->getParameterValue('address'));
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
