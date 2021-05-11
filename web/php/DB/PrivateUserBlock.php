<?php

namespace Wikidot\DB;


use Wikijump\Models\User;

/**
 * Object Model Class.
 *
 */
class PrivateUserBlock extends PrivateUserBlockBase
{

    public function getBlockedUser()
    {
        return User::find($this->getBlockedUserId());
    }
}
