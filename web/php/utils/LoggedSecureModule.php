<?php

namespace Wikidot\Utils;

abstract use Ozone\Framework\SmartyModule;

class LoggedSecureModule extends SmartyModule
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
            return false;
        }
    }
}
