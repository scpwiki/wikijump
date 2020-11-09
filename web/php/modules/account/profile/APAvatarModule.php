<?php
class APAvatarModule extends AccountBaseModule
{

    public function build($runData)
    {

        $user = $runData->getUser();
        $userId = $user->getUserId();

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
        }

        $runData->contextAdd("hasAvatar", $hasAvatar);
    }
}
