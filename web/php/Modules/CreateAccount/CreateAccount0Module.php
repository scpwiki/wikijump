<?php

namespace Wikidot\Modules\CreateAccount;


use Ozone\Framework\SmartyModule;
use Wikidot\Utils\CryptUtils;
use Wikidot\Utils\GlobalProperties;
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

        $runData->ajaxResponseAdd("key", CryptUtils::modulus()); // TODO: what does this do?
        $runData->contextAdd('captchaSiteKey', GlobalProperties::$FR_CAPTCHA_SITE_KEY);
        $runData->sessionAdd("rstep", 0);
        $this->extraJs[] = '/common--javascript/crypto/rsa.js';
    }
}
