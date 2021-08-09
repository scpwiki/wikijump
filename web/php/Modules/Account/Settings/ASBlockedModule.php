<?php

namespace Wikidot\Modules\Account\Settings;

use Ozone\Framework\Database\Criteria;
use Wikidot\Utils\AccountBaseModule;
use Wikijump\Models\User;

class ASBlockedModule extends AccountBaseModule
{

    /**
     * Retrieve all users this user has blocked.
     * @param $runData
     */
    public function build($runData)
    {
        /** @var User $user */
        $user = $runData->getUser();
        $blocks = $user->viewBlockedUsers();

        if ($blocks->count() > 0) {
            $runData->contextAdd("blocks", $blocks);
        }
    }
}
