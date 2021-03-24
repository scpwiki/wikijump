<?php

namespace Wikidot\Modules\CreateAccount;


use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;

class CreateAccount2Module extends SmartyModule
{

    public function isAllowed($runData)
    {
        if ($runData->getUserId() !== null) {
            throw new ProcessException(_("You are already logged in. Why would you want to create a new account?"), "logged_in");
        }
        $rstep = $runData->sessionGet("rstep");
        return true;
    }

    public function build($runData)
    {

        $runData->contextAdd('evcode', $runData->sessionGet('evcode'));
        $runData->sessionAdd("rstep", 2);

        $data = $runData->sessionGet("ca_data");
        $email = $data['email'];
        $name = $data['name'];

        $runData->contextAdd("email", $email);
        $runData->contextAdd("name", $name);
    }
}
