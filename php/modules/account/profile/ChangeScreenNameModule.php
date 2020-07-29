<?php
class ChangeScreenNameModule extends AccountBaseModule
{

    public function build($runData)
    {

        $user = $runData->getUser();
        $userId = $user->getUserId();

        $runData->contextAdd('user', $user);
        $runData->contextAdd('profile', $user->getProfile());
    }
}
