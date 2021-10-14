<?php

declare(strict_types=1);

namespace Wikijump\Policies;

use Illuminate\Auth\Access\HandlesAuthorization;
use Illuminate\Auth\Access\Response;
use Wikijump\Models\User;
use Wikijump\Models\UserMessage;

/**
 * Class UserMessagePolicy
 * @package Wikijump\Policies
 */
class UserMessagePolicy
{
    use HandlesAuthorization;

    /**
     * Global allow and deny for User Policies.
     * @see https://laravel.com/docs/8.x/authorization#policy-filters
     * @param User $user
     * @param string $ability
     * @return bool|null
     */
    public function before(User $user, string $ability): ?bool
    {
        /** The farm admin account can do anything. */
        if ($user->id === User::ADMIN_USER) {
            return true;
        }

        /** And a globally banned user can't do anything. */
        if ($user->isBanned()) {
            return false;
        }

        return null;
    }

    /**
     * Determine whether the user can send the message.
     *
     * @param User $sender
     * @param UserMessage $userMessage
     * @return Response
     */
    public function send(User $sender, UserMessage $userMessage): Response
    {
        $recipient = $userMessage->recipient;

        /** If the recipient has the `allow_pms` setting disabled, deny. */
        if ($recipient->get('allow_pms') === false) {
            return Response::deny(_('The recipient does not accept private messages.'));
        }

        /** The sender can't send messages if they're blocking or blocked by the recipient. */
        if ($sender->isBlockingUser($recipient)) {
            return Response::deny(_('You are blocking the recipient.'));
        }
        if ($sender->isBlockedByUser($recipient)) {
            return Response::deny(_('You are blocked by the recipient.'));
        }

        /** Users can always message their contacts. */
        if ($sender->isContact($recipient)) {
            return Response::allow();
        }

        /**
         * Stub for a later check of if users share a wiki.
         * TODO: Make this user method as part of site membership refactoring.
         */
        # if($sender->sharesSiteWith($recipient)) { Response::allow(); }

        /**
         * Otherwise, they are allowed. We'll probably change this to a deny once
         *  the shared site logic is in place, and add a setting to allow from all users.
         */
        return Response::allow();
    }
}
