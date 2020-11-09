<?php
use DB\UserSettingsPeer;

class ASMessagesModule extends AccountBaseModule
{

    public function build($runData)
    {
        $us = UserSettingsPeer::instance()->selectByPrimaryKey($runData->getUserId());
        $runData->contextAdd("from", trim($us->getReceivePm()));
    }
}
