<?php

namespace Wikidot\Modules\Account\Settings;




use Wikidot\Utils\AccountBaseModule;

class ASEmailModule extends AccountBaseModule
{

    public function build($runData)
    {
        $user = $runData->getUser();
        $email = $user->email;

        $runData->contextAdd("email", $email);
    }
}
