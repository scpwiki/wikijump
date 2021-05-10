<?php

namespace Wikidot\Modules\Account\Settings;




use Wikidot\Utils\AccountBaseModule;

class ASLanguageModule extends AccountBaseModule
{

    public function build($runData)
    {
        $user = $runData->getUser();
        $runData->contextAdd("lang", $user->language);
    }
}
