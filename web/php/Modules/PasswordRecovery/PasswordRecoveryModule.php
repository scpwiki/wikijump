<?php

namespace Wikidot\Modules\PasswordRecovery;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\CryptUtils;
use Wikidot\Utils\ProcessException;

class PasswordRecoveryModule extends SmartyModule
{

    public function build($runData)
    {
        $userId = $runData->getUserId();
        if ($userId !== null) {
            throw new ProcessException(_("You already are logged in."), "already_logged");
        }
        $runData->ajaxResponseAdd("key", CryptUtils::modulus());

        $runData->sessionStart();
        $seed = CryptUtils::generateSeed(10);
        $runData->sessionAdd("login_seed", $seed);
        $this->extraJs[] = '/common--javascript/crypto/rsa.js';
    }
}
