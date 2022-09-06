<?php
declare(strict_types=1);

namespace Wikidot\Modules\Account\Contacts;

use Wikidot\Utils\AccountBaseModule;
use Wikijump\Models\User;

/**
 * Module for retrieving contacts.
 * @package Wikidot\Modules\Account\Contacts
 */
class AccountContactsModule extends AccountBaseModule
{
    /**
     * Builds a list of this user's contacts.
     * @param $runData
     */
    public function build($runData)
    {
        /** @var User $user */
        $user = $runData->getUser();

        $runData->contextAdd('back', $user->contacts());
        $runData->contextAdd('countBack', $user->contacts()->count());
        $runData->contextAdd('contacts', $user->contacts()->sortBy('username'));

        /** I think this is pagination. */
        $runData->contextAdd('maxContacts', 50);
    }
}
