<?php

namespace Wikidot\Modules\UserInfo;


use Wikidot\Utils\ProcessException;
use Wikidot\Utils\SmartyLocalizedModule;
use Wikidot\Utils\WDPermissionException;
use Wikijump\Models\User;

class UserAddToContactsModule extends SmartyLocalizedModule
{

    public function isAllowed($runData)
    {
        $userId = $runData->getUserId();
        if(!$userId) {
            throw new WDPermissionException(_("You should login first."));
        }
        return true;
    }

    public function build($runData)
    {
        $pl = $runData->getParameterList();

        $targetUserId = $pl->getParameterValue("userId");

        $targetUser = User::find($targetUserId);

        if ($targetUser == null) {
            throw new ProcessException(_("User cannot be found."), "no_user");
        }

        $runData->contextAdd("user", $targetUser);
    }
}
