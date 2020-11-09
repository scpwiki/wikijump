<?php
use DB\AdminPeer;
use DB\OzoneUserPeer;

class ManageSiteDeleteModule extends ManageSiteBaseModule
{

    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $user = $runData->getUser();
        $runData->contextAdd("site", $site);

        $c = new Criteria();
        $c->add("user_id", $user->getUserId());
        $c->add("site_id", $site->getSiteId());
        $c->add("founder", true);
        $rel = AdminPeer::instance()->selectOne($c);

        if ($rel) {
            $runData->contextAdd('allowed', true);
        } else {
            $c = new Criteria();
            $c->add("site_id", $site->getSiteId());
            $c->add("founder", true);
            $f = AdminPeer::instance()->selectOne($c);
            $founder = OzoneUserPeer::instance()->selectByPrimaryKey($f->getUserId());
            $runData->contextAdd('founder', $founder);
        }
    }
}
