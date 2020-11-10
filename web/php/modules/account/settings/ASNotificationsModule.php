<?php
class ASNotificationsModule extends AccountBaseModule
{

    public function build($runData)
    {
        $user = $runData->getUser();
        $username = $user->getName();

        $password = $user->getPassword();

        $password = substr($password, 0, 15);

        $runData->contextAdd("feedUsername", $username);
        $runData->contextAdd("feedPassword", $password);

        $runData->contextAdd("settings", $user->getSettings());
    }
}
