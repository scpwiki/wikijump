<?php

namespace Wikidot\Modules\Account;


use Illuminate\Support\Facades\Auth;
use Wikidot\Utils\AccountBaseModule;

class AccountWelcomeModule extends AccountBaseModule
{

    public function build($runData)
    {

        $user = $runData->getUser() ?? Auth::user();
        $runData->contextAdd("user", $user);

        $userId = $user->id;

        $tips = array();

        $avatarUri = '/user--avatar/'.$userId;

        $runData->contextAdd("avatarUri", $avatarUri);

        $runData->contextAdd("hasAvatar", true);
        if (count($tips)>0) {
            $runData->contextAdd("tips", $tips);
        }
    }
}
