<?php
declare(strict_types=1);

namespace Wikidot\Actions;
use Ozone\Framework\SmartyAction;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionException;
use Wikijump\Models\User;

/**
 * Event handler for contacts.
 * @package Wikidot\Actions
 */
class ContactsAction extends SmartyAction
{

    /**
     * Check if this user is allowed to add a contact.
     * @param $runData
     * @return bool
     * @throws WDPermissionException
     */
    public function isAllowed($runData): bool
    {
        if(!$runData->getUserId()) {
            throw new WDPermissionException(_('Not allowed. You should login first.'));
        }
        return true;
    }

    /**
     * Stub class to fulfill Action contract.
     * @param $runData
     */
    public function perform($runData)
    {
    }

    /**
     * Add a user to this user's contacts. Bidirectional.
     * @param $runData
     * @throws ProcessException
     */
    public function addContactEvent($runData)
    {
        $pl = $runData->getParameterList();
        $target_user = User::find($pl->getParameterValue('userId'));
        /** @var User $user */
        $user = $runData->getUser();

        if ($target_user === null) {
            throw new ProcessException(_('User cannot be found.'), 'no_user');
        }

        if ($target_user->id === $user->id) {
            throw new ProcessException(_('You cannot add yourself to your contacts.'), 'not_yourself');
        }


        if ($user->isContact($target_user)) {
            throw new ProcessException(_('This user is already in your contacts.'), 'already_contact');
        }

        if ($user->contacts()-count() >= config('wikijump.contact_limit')) {
            throw new ProcessException(_('You cannot add any more contacts.'), 'max_reached');
        }

        $user->addContact($target_user);
    }

    /**
     * Remove a user from this user's contacts. Bidirectional.
     * @param $runData
     * @throws ProcessException
     */
    public function removeContactEvent($runData)
    {
        $pl = $runData->getParameterList();
        /** @var User $user */
        $user = $runData->getUser();
        $target_user = User::find($pl->getParameterValue('userId'));

        if ($target_user === null) {
            throw new ProcessException(_('No user found.'), 'no_user');
        }

        $user->removeContact($target_user);
    }
}
