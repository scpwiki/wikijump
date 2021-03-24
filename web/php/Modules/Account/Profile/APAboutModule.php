<?php

namespace Wikidot\Modules\Account\Profile;




use Wikidot\DB\ProfilePeer;
use Wikidot\Utils\AccountBaseModule;

class APAboutModule extends AccountBaseModule
{

    public function build($runData)
    {

        $userId = $runData->getUserId();
        $profile = ProfilePeer::instance()->selectByPrimaryKey($userId);

        $runData->contextAdd("profile", $profile);
    }
}
