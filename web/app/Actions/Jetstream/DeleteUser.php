<?php

declare(strict_types=1);

namespace Wikijump\Actions\Jetstream;

use Laravel\Jetstream\Contracts\DeletesUsers;

/**
 * All-in-one class for removing a user's avatar, API keys, and the user object itself.
 * @package Wikijump\Actions\Jetstream
 */
class DeleteUser implements DeletesUsers
{
    /**
     * Delete the given user.
     *
     * @param  mixed  $user
     * @return void
     */
    public function delete($user)
    {
        $user->deleteAvatar();
        $user->tokens->each->delete();
        $user->delete();
    }
}
