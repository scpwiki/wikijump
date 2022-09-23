<?php

namespace Wikidot\DB;


use Wikijump\Models\User;

/**
 * Object Model Class.
 *
 */
class EmailInvitation extends EmailInvitationBase
{

    public function getUser()
    {
        return User::find($this->getUserId());
    }
}
