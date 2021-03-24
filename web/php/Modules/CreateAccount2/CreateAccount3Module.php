<?php

namespace Wikidot\Modules\CreateAccount2;


use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;

class CreateAccount3Module extends SmartyModule
{

    public function build($runData)
    {
        $user = $runData->getUser();
        if (!$user) {
            throw new ProcessException(_('No valid user found - account creation failed.'));
        }
        $runData->contextAdd("user", $user);
        $pl = $runData->getParameterList();

        $originalUrl = $pl->getParameterValue('origUrl');
        $runData->contextAdd('originalUrl', $originalUrl);
        $runData->contextAdd('originalUrlStripped', preg_replace(';^https?://;', '', $originalUrl));
    }
}
