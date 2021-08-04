<?php
declare(strict_types=1);

namespace Wikidot\Modules\Account\PM;

use Wikidot\Utils\AccountBaseModule;
use Wikijump\Models\User;

/**
 * Choose contacts to send a PM to.
 * @package Wikidot\Modules\Account\PM
 */
class PMComposeContactsListModule extends AccountBaseModule
{

    /**
     * Build the list of contacts.
     */
    public function build($runData)
    {
        /** @var User $user */
        $user = $runData->getUser();

        $contacts = $user->contacts()->sortBy('username');

        $runData->contextAdd('contacts', $contacts);
    }
}
