<?php

namespace Wikidot\Modules\Report;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\PageAbuseFlagPeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\WDPermissionException;

class FlagPageModule extends SmartyModule
{

    public function isAllowed($runData)
    {
        $userId = $runData->getUserId();
        if ($userId === null) {
            throw new WDPermissionException(_("This option is available only to registered (and logged-in) users."));
        }

        return true;
    }

    public function build($runData)
    {
        $pl = $runData->getParameterList();

        $page = $runData->getTemp("page");
        $site = $runData->getTemp("site");
        $user = $runData->getUser();
        // check if flagged already
        $c = new Criteria();
        $c->add("user_id", $user->id);
        $c->add("site_id", $site->getSiteId());
        $c->add("path", $path);

        $flag = PageAbuseFlagPeer::instance()->selectOne($c);

        if ($flag) {
            $runData->contextAdd("flagged", true);
        }
    }
}
