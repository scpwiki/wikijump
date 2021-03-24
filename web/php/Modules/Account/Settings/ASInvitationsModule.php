<?php

namespace Wikidot\Modules\Account\Settings;




use Wikidot\DB\UserSettingsPeer;
use Wikidot\Utils\AccountBaseModule;

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
