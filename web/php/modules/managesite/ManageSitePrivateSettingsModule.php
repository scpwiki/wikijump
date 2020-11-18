<?php
use DB\OzoneUserPeer;

class ManageSitePrivateSettingsModule extends ManageSiteBaseModule
{

    protected $processPage = true;

    public function build($runData)
    {

        $site = $runData->getTemp("site");
        $runData->contextAdd("site", $site);
        $runData->contextAdd("settings", $site->getSettings());
        $runData->contextAdd("superSettings", $site->getSuperSettings());

        // get the viewers
        $c = new Criteria();
        $q = "SELECT ozone_user.* FROM ozone_user, site_viewer WHERE site_viewer.site_id='".$site->getSiteId()."' " .
                "AND ozone_user.user_id = site_viewer.user_id ORDER BY ozone_user.nick_name";
        $c->setExplicitQuery($q);

        $viewers = OzoneUserPeer::instance()->select($c);

        $runData->contextAdd("viewers", $viewers);
        $runData->contextAdd("settings", $site->getSettings());
    }
}
