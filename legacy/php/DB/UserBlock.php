<?php

namespace Wikidot\DB;


use Wikijump\Models\User;

/**
 * Object Model Class.
 *
 */
class UserBlock extends UserBlockBase
{

    public function getUser()
    {
        return User::find($this->getUserId());
    }
}
