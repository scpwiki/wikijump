<?php

namespace Wikidot\Modules\Account\Settings;




use Wikidot\Utils\AccountBaseModule;
use Wikijump\Models\User;

class ASMessagesModule extends AccountBaseModule
{

    public function build($runData)
    {
        $user = User::find($runData->getUserId());
        $runData->contextAdd("from", $user->get('receive_pm'));
    }
}
