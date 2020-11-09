<?php
use DB\DomainRedirectPeer;

class ManageSiteDomainModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        $runData->contextAdd("site", $site);

        // get redirects

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->addOrderAscending("url");

        $redirects = DomainRedirectPeer::instance()->select($c);
        $ra = array();
        foreach ($redirects as $r) {
            $ra[] = $r->getUrl();
        }

        $runData->contextAdd("redirects", $ra);
    }
}
