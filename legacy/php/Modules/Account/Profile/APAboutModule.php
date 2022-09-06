<?php

namespace Wikidot\Modules\Account\Profile;




use Wikidot\Utils\AccountBaseModule;
use Wikijump\Models\User;

class APAboutModule extends AccountBaseModule
{

    public function build($runData)
    {

        $userId = $runData->getUserId();
        $profile = User::find($userId);

        $runData->contextAdd("profile", $profile);
    }
}
