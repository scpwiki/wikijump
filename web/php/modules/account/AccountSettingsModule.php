<?php
class AccountSettingsModule extends AccountBaseModule
{

    public function build($runData)
    {

        $user = $runData->getUser();
        $runData->contextAdd("user", $user);
    }
}
