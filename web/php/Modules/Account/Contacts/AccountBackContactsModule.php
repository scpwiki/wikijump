<?php
declare(strict_types=1);

namespace Wikidot\Modules\Account\Contacts;

use Wikidot\Utils\AccountBaseModule;
use Wikijump\Models\User;

/**
 * Retrieves "back" contacts, a legacy behavior. To be removed.
 * @package Wikidot\Modules\Account\Contacts
 */
class AccountBackContactsModule extends AccountBaseModule
{

    /**
     * Build the "back" contacts list. This used to be unidirectional, now it's bidirectional.
     */
    public function build($runData)
    {
        /** @var User $user */
        $user = $runData->getUser();

        $runData->contextAdd('contacts', $user->contacts()->sortBy('username'));
    }
}
