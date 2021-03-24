<?php

namespace Wikidot\Utils;

use Ozone\Framework\SmartyScreen;

abstract class LoggedSecureScreen extends SmartyScreen
{

    public function isAllowed($runData)
    {
        if ($runData->isUserAuthenticated()) {
            $runData->contextAdd("loggedUserId", $runData->getUserId());
            return true;
        } else {
            // if some conditions are met, you can be succesfuly redirected after the login
            if ($runData->getRequestMethod() == "GET" || $runData->getAction === null) {
                $runData->contextAdd("requestedTemplate", $runData->getScreenTemplate());
                $rp = $runData->getParameterList()->asArray();
                unset($rp['template']);
                $runData->contextAdd("requestedParameters", serialize($rp));
            }
            $runData->setScreenTemplate("LoginUser");
            return false;
        }
    }
}
