<?php
use DB\OzoneUserPeer;

class UserAddToContactsModule extends SmartyLocalizedModule
{

    public function isAllowed($runData)
    {
        $userId = $runData->getUserId();
        if ($userId == null || $userId <1) {
            throw new WDPermissionException(_("You should login first."));
        }
        return true;
    }

    public function build($runData)
    {
        $pl = $runData->getParameterList();

        $targetUserId = $pl->getParameterValue("userId");

        $targetUser = OzoneUserPeer::instance()->selectByPrimaryKey($targetUserId);

        if ($targetUser == null) {
            throw new ProcessException(_("User can not be found."), "no_user");
        }

        // check how many contacts so far...

        $runData->contextAdd("user", $targetUser);
    }
}
