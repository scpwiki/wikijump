<?php

namespace Wikidot\Modules\Login;

use Ozone\Framework\SmartyModule;

class LoginStatusModule2 extends SmartyModule
{

    public function build($runData)
    {
        $user = $runData->getUser();
        if ($user) {
            $userName = $user->getName();
            $nick = $user->getNickName();

            $runData->contextAdd("nick", $nick);
        }
        $runData->contextAdd("authenticated", $authenticated);
    }
}
