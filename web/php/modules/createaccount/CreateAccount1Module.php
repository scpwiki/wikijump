<?php
class CreateAccount1Module extends SmartyModule
{

    public function isAllowed($runData)
    {
        if ($runData->getUserId() !== null) {
            throw new ProcessException(_("You are already logged in. Why would you want to create a new account?"), "logged_in");
        }
        $rstep = $runData->sessionGet("rstep");
        if ($rstep === null || !($rstep == 1 || $rstep == 0 || $rstep == 2)) {
            throw new ProcessException(_("Registration flow error."), "registration_error");
        }
        return true;
    }

    public function build($runData)
    {

        $runData->sessionAdd("rstep", 1);
    }
}
