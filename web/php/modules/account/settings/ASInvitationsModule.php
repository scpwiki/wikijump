<?php
use DB\UserSettingsPeer;

class ASInvitationsModule extends AccountBaseModule
{

    public function build($runData)
    {
        $us = UserSettingsPeer::instance()->selectByPrimaryKey($runData->getUserId());
        if ($us->getReceiveInvitations()) {
            $runData->contextAdd("receiveInvitations", true);
        }
    }
}
