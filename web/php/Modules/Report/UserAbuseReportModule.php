<?php

namespace Wikidot\Modules\Report;

use Wikidot\DB\OzoneUserPeer;

use Ozone\Framework\SmartyModule;

class UserAbuseReportModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $userId = $pl->getParameterValue("userId");

        $user = OzoneUserPeer::instance()->selectByPrimaryKey($userId);
        $runData->contextAdd("user", $user);

        $site =  $runData->getTemp("site");
        if ($site->getUnixName() !== 'www') {
            $runData->contextAdd("site", $site);
        }
    }
}
