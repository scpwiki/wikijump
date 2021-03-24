<?php

namespace Wikidot\Modules\Account\Settings;




use Wikidot\DB\UserSettingsPeer;
use Wikidot\Utils\AccountBaseModule;

class ASMessagesModule extends AccountBaseModule
{

    public function build($runData)
    {
        $us = UserSettingsPeer::instance()->selectByPrimaryKey($runData->getUserId());
        $runData->contextAdd("from", trim($us->getReceivePm()));
    }
}
