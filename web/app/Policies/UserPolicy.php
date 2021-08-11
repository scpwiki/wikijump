<?php

declare(strict_types=1);

namespace Wikijump\Policies;

use Illuminate\Auth\Access\HandlesAuthorization;
use Illuminate\Auth\Access\Response;
use Wikijump\Models\User;

/**
 * Authorization Policy for User models.
 * @package Wikijump\Policies
 */
class UserPolicy
{
    use HandlesAuthorization;

    /**
     * Global allow and deny for User Policies.
     * @see https://laravel.com/docs/8.x/authorization#policy-filters
     * @param User $user
     * @param string $ability
     * @return bool|void
     */
    public function before(User $user, string $ability) : ?bool
    {
        /** The farm admin account can do anything. */
        if($user->id == 1) { return true; }

        /** And a globally banned user can't do anything. */
        if($user->isBanned()) { return false; }
    }

    /**
     * Determine whether the user can send a private message to another user.
     * @param User $sender
     * @param User $recipient
     * @return Response
     */
    public function message(User $sender, User $recipient) : Response
    {
        /** If the recipient has the `allow_pms` setting disabled, deny. */
        if($recipient->get('allow_pms') === false)
        {
            Response::deny(_('The recipient does not accept private messages.'));
        }

        /** The sender can't send messages if they're blocking or blocked by the recipient. */
        if($sender->isBlockingUser($recipient))
        {
            Response::deny(_('You are blocking the recipient.'));
        }
        if($sender->isBlockedByUser($recipient))
        {
            Response::deny(_('You are blocked by the recipient.'));
        }

        /** Users can always message their contacts. */
        if($sender->isContact($recipient)) { Response::allow(); }

        /**
         * Stub for a later check of if users share a wiki.
         * TODO: Make this user method as part of site membership refactoring.
         */
        # if($sender->sharesSiteWith($recipient)) { Response::allow(); }

        /**
         * Otherwise, they are allowed. We'll probably change this to a deny once
         *  the shared site logic is in place, and add a setting to allow from all users.
         */
        Response::allow();

    }

    /**
     * Determine whether the user can view any models.
     *
     * @param User $user
     * @return Response|bool
     */
    public function viewAny(User $user)
    {
        //
    }

    /**
     * Determine whether the user can view the model.
     *
     * @param User $user
     * @param User $model
     * @return Response|bool
     */
    public function view(User $user, User $model)
    {
        //
    }

    /**
     * Determine whether the user can create models.
     *
     * @param User $user
     * @return Response|bool
     */
    public function create(User $user)
    {
        //
    }

    /**
     * Determine whether the user can update the model.
     *
     * @param User $user
     * @param User $model
     * @return Response|bool
     */
    public function update(User $user, User $model)
    {
        //
    }

    /**
     * Determine whether the user can delete the model.
     *
     * @param User $user
     * @param User $model
     * @return Response|bool
     */
    public function delete(User $user, User $model)
    {
        //
    }

    /**
     * Determine whether the user can restore the model.
     *
     * @param User $user
     * @param User $model
     * @return Response|bool
     */
    public function restore(User $user, User $model)
    {
        //
    }

    /**
     * Determine whether the user can permanently delete the model.
     *
     * @param User $user
     * @param User $model
     * @return Response|bool
     */
    public function forceDelete(User $user, User $model)
    {
        //
    }
}
