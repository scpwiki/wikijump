<?php

namespace Wikidot\Modules\ManageSite;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\AdminPeer;

use Wikidot\Utils\ManageSiteBaseModule;
use Wikijump\Models\User;

class ManageSiteRenameModule extends ManageSiteBaseModule
{

    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $user = $runData->getUser();
        $runData->contextAdd("site", $site);

        $c = new Criteria();
        $c->add("user_id", $user->id);
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
            $founder = User::find($f->getUserId());
            $runData->contextAdd('founder', $founder);
        }
    }
}
