<?php
class LoginModule2 extends SmartyModule
{

    public function build($runData)
    {
        // check if not already logged in...

        $user = $runData->getUser();
        if ($user) {
            throw new ProcessException(_("You already are logged in."), "already_logged");
        }
    }
}
