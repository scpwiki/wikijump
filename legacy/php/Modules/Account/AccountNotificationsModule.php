<?php

namespace Wikidot\Modules\Account;


use Wikidot\Utils\AccountBaseModule;

class AccountNotificationsModule extends AccountBaseModule
{

    public function build($runData)
    {
        $user = $runData->getUser();
        $username = $user->username;

        $password = $user->password;

        $runData->contextAdd("feedUsername", $username);
        $runData->contextAdd("feedPassword", $password);
    }
}
