<?php
use DB\OpenidEntryPeer;

class ManageSiteOpenIDModule extends ManageSiteBaseModule
{

    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $settings = $site->getSettings();

        $runData->contextAdd("siteDomain", $site->getDomain());

        $openIdServices = array(
            array(  'pattern' => '^[a-z0-9\.\-]+\.myopenid\.com\/?$',
                    'server' => 'http://www.myopenid.com/server'),
            array(  'pattern' => '^[a-z0-9\.\-]+\.getopenid\.com\/?$',
                    'server' => 'https://getopenid.com/server'),
            array(  'pattern' => '^[a-z0-9\.\-]+\.livejournal\.com\/?$',
                    'server' => 'http://www.livejournal.com/openid/server.bml'),
            array(  'pattern' => '^[a-z0-9\.\-]+\.vox\.com\/?$',
                    'server' => 'http://www.vox.com/openid/server'),
            array(  'pattern' => '^[a-z0-9\.\-]+\.verisignlabs\.com\/?$',
                    'server' => 'https://pip.verisignlabs.com/server'),
            array(  'pattern' => '^[a-z0-9\.\-]+\.openid\.pl\/?$',
                    'server' => 'http://openid.pl/server'),
                    array(  'pattern' => '^myid\.pl\/id\/',
                    'server' => 'http://myid.pl/auth')
        );

        $json = new JSONService();
        $os = $json->encode($openIdServices);

        $runData->contextAdd("openIdServices", $os);

        // current settings
        $runData->contextAdd("enabled", $settings->getOpenidEnabled());

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("page_id", null);
        $ooroot = OpenidEntryPeer::instance()->selectOne($c);

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("page_id", null, "!=");
        $oos = OpenidEntryPeer::instance()->select($c);

        $runData->contextAdd("openIdRoot", $ooroot);

        $runData->contextAdd("openIds", $oos);
    }
}
