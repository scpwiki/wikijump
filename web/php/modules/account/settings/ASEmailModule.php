<?php
class ASEmailModule extends AccountBaseModule
{

    public function build($runData)
    {
        $user = $runData->getUser();
        $email = $user->getEmail();

        $runData->contextAdd("email", $email);
    }
}
