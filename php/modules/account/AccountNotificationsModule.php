<?php
class AccountNotificationsModule extends AccountBaseModule
{

    public function build($runData)
    {
        $user = $runData->getUser();
        $username = $user->getName();

        $password = $user->getPassword();

        $runData->contextAdd("feedUsername", $username);
        $runData->contextAdd("feedPassword", $password);
    }
}
