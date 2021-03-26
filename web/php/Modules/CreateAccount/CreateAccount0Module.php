<?php

namespace Wikidot\Modules\CreateAccount;


use Ozone\Framework\SmartyModule;
use Wikidot\Utils\CryptUtils;
use Wikidot\Utils\ProcessException;

class CreateAccount0Module extends SmartyModule
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

        $code =  $runData->sessionGet('captchaCode');

        $runData->ajaxResponseAdd("key", CryptUtils::modulus());

        if ($code === null) {
            srand((double)microtime()*1000000);
            $string = md5(rand(0, 9999));
            $code = substr($string, 2, 4);
            $code = str_replace('0', 'O', $code);
            $code = strtoupper($code);
            $runData->sessionAdd("captchaCode", $code);
        }

        $runData->contextAdd('captchaSiteKey', 'FCMT74J3ULGV4PI8'); // TODO: get this dynamically based on environment (dev vs prod)

        $runData->sessionAdd("rstep", 0);
        $this->extraJs[] = '/common--javascript/crypto/rsa.js';
    }
}
