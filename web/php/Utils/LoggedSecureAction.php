<?php

namespace Wikidot\Utils;

use Ozone\Framework\SmartyAction;

abstract class LoggedSecureAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        if ($runData->isUserAuthenticated()) {
            return true;
        } else {
            $runData->setScreenTemplate("LoginUser");
            return false;
        }
    }
}
