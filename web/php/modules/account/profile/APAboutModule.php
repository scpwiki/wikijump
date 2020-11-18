<?php
use DB\ProfilePeer;

class APAboutModule extends AccountBaseModule
{

    public function build($runData)
    {

        $userId = $runData->getUserId();
        $profile = ProfilePeer::instance()->selectByPrimaryKey($userId);

        $runData->contextAdd("profile", $profile);
    }
}
