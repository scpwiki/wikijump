<?php

namespace Wikidot\Modules\Account\Watch;




use Wikidot\Utils\AccountBaseModule;

class AWFeedModule extends AccountBaseModule
{

    public function build($runData)
    {
        $user = $runData->getUser();
        $username = $user->getName();

        $password = $user->getPassword();

        $password = substr($password, 0, 15);

        $runData->contextAdd("feedUsername", $username);
        $runData->contextAdd("feedPassword", $password);
    }
}
