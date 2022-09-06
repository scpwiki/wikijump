<?php

namespace Wikidot\Modules\Account\Settings;




use Wikidot\Utils\AccountBaseModule;

class ASNotificationsModule extends AccountBaseModule
{

    public function build($runData)
    {
        $user = $runData->getUser();
        $username = $user->username;

        $password = $user->password;

        $password = substr($password, 0, 15);

        $runData->contextAdd("feedUsername", $username);
        $runData->contextAdd("feedPassword", $password);

        $runData->contextAdd("settings", $user->getSettings());
    }
}
