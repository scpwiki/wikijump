<?php

namespace Wikidot\Modules\Account\Settings;




use Wikidot\Utils\AccountBaseModule;
use Wikijump\Models\User;

class ASInvitationsModule extends AccountBaseModule
{

    public function build($runData)
    {
        $user = User::find($runData->getUserId());
        if($user->get('receive_invitations') === true) {
            $runData->contextAdd("receiveInvitations", true);
        }
    }
}
