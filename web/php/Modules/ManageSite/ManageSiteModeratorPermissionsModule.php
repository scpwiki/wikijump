<?php

namespace Wikidot\Modules\ManageSite;

use Wikidot\DB\ModeratorPeer;
use Wikidot\Utils\ManageSiteBaseModule;
use Wikidot\Utils\ProcessException;

class ManageSiteModeratorPermissionsModule extends ManageSiteBaseModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $moderatorId = $pl->getParameterValue("moderatorId");
        $site = $runData->getTemp("site");
        $mod = ModeratorPeer::instance()->selectByPrimaryKey($moderatorId);
        if ($mod == null || $mod->getSiteId() != $site->getSiteId()) {
            throw new ProcessException("No such moderator.");
        }
        $runData->contextAdd("moderator", $mod);
        $ps = $mod->getPermissions();

        if (strpos($ps, 'p')!== false) {
            $runData->contextAdd("ppages", true);
        }
        if (strpos($ps, 'f')!== false) {
            $runData->contextAdd("pforum", true);
        }
        if (strpos($ps, 'u')!== false) {
            $runData->contextAdd("pusers", true);
        }

        $runData->ajaxResponseAdd("moderatorId", $moderatorId);
    }
}
