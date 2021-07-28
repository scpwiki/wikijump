<?php

namespace Wikidot\Modules\CreateAccount;


use Ozone\Framework\SmartyModule;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\ProcessException;

class CreateAccountStep1Module extends SmartyModule
{

    public function isAllowed($runData)
    {
        if ($runData->getUserId() !== null) {
            throw new ProcessException(_("You are already logged in."), "logged_in");
        }

        $rstep = $runData->sessionGet("rstep");
        return true;
    }

    public function build($runData)
    {

        $site = $runData->getTemp('site');
        // check the connection type
        if (!$_SERVER['HTTPS'] && $site->getSettings()->getSslMode() && !$runData->getParameterList()->getParameterValue('disableSSL')) {
            // not enabled, redirect to http:
            $site = $runData->getTemp("site");
            header("HTTP/1.1 301 Moved Permanently");
            header("Location: ".'https://'.$site->getDomain().$_SERVER['REQUEST_URI']);
            exit();
        }

        $runData->contextAdd('captchaSiteKey', GlobalProperties::$FR_CAPTCHA_SITE_KEY);
        $runData->sessionAdd('rstep', 0);

        $pl = $runData->getParameterList();
        $originalUrl = $pl->getParameterValue('origUrl');
        if ($originalUrl) {
            $originalUrlForce = $pl->getParameterValue('origUrlForce');
            if ($originalUrlForce) {
                $runData->sessionAdd('loginOriginalUrlForce', true);
            }
            $runData->sessionAdd('loginOriginalUrl', $originalUrl);
        }
    }
}
