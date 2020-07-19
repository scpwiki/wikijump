<?php
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

        $code =  $runData->sessionGet('captchaCode');

        if ($code === null) {
            srand((double)microtime()*1000000);
            $string = md5(rand(0, 9999));
            $code = substr($string, 2, 4);
            $code = str_replace('0', 'O', $code);
            $code = strtoupper($code);
            $runData->sessionAdd("captchaCode", $code);
        }
        $runData->contextAdd('evcode', $code);
        $runData->contextAdd("rand", rand(0, 1000));

        $runData->sessionAdd("rstep", 0);

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
