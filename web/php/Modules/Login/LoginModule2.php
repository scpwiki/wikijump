<?php

namespace Wikidot\Modules\Login;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;

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
