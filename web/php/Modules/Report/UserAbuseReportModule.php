<?php

namespace Wikidot\Modules\Report;



use Ozone\Framework\SmartyModule;
use Wikijump\Models\User;

class UserAbuseReportModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $userId = $pl->getParameterValue("userId");

        $user = User::find($userId);
        $runData->contextAdd("user", $user);

        $site =  $runData->getTemp("site");
        if ($site->getUnixName() !== 'www') {
            $runData->contextAdd("site", $site);
        }
    }
}
