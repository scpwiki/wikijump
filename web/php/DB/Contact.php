<?php

namespace Wikidot\DB;


use Wikijump\Models\User;

/**
 * Object Model Class.
 *
 */
class Contact extends ContactBase
{

    public function getUser()
    {
        return User::find($this->getUserId());
    }

    public function getTargetUser()
    {
        return User::find($this->getTargetUserId());
    }
}
