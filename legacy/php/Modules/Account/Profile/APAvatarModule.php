<?php

namespace Wikidot\Modules\Account\Profile;




use Wikidot\Utils\AccountBaseModule;

class APAvatarModule extends AccountBaseModule
{

    public function build($runData)
    {

        $user = $runData->getUser();
        $userId = $user->id;
        $avatarUri = '/user--avatar/'.$userId;

        $runData->contextAdd("avatarUri", $avatarUri);

        $runData->contextAdd("hasAvatar", true);
    }
}
