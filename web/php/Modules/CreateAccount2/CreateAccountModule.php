<?php

namespace Wikidot\Modules\CreateAccount2;


use Ozone\Framework\SmartyModule;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\ProcessException;

class CreateAccountModule extends SmartyModule
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

        $site = $runData->getTemp('site');
        // check the connection type
        if (!$_SERVER['HTTPS'] && $site->getSettings()->getSslMode() && !$runData->getParameterList()->getParameterValue('disableSSL')) {
            // not enabled, redirect to http:
            $site = $runData->getTemp("site");
            header("HTTP/1.1 301 Moved Permanently");
            header("Location: ".'https://'.$site->getDomain().$_SERVER['REQUEST_URI']);
            exit();
        }

        // some random value wikidot wants to know you're in a flow
        $evcode = md5(random_int(0, 10000));

        $runData->contextAdd('captchaSiteKey', GlobalProperties::$FR_CAPTCHA_SITE_KEY);
        $runData->contextAdd('evcode', $evcode);
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
