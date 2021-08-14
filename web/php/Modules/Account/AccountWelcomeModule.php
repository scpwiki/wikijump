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

        // check if has an avatar
        $avatarDir = WIKIJUMP_ROOT.'/web/files--common/images/avatars/';
        $avatarDir .= '' . floor($userId/1000).'/'.$userId;
        $avatarPath = $avatarDir."/a48.png";
        if (file_exists($avatarPath)) {
            $hasAvatar = true;
            $avatarUri = '/common--images/avatars/'.floor($userId/1000).'/'.$userId.'/a48.png';
            $avatarUri .= '?'.rand(1, 10000);
            $runData->contextAdd("avatarUri", $avatarUri);
        } else {
            $hasAvatar = false;
            $tips['avatar'] = true;
        }

        $runData->contextAdd("hasAvatar", $hasAvatar);
        if (count($tips)>0) {
            $runData->contextAdd("tips", $tips);
        }
    }
}
