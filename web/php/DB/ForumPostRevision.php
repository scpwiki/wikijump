<?php

namespace Wikidot\DB;


use Wikijump\Models\User;

/**
 * Object Model Class.
 *
 */
class ForumPostRevision extends ForumPostRevisionBase
{

    public function getUser()
    {
        if ($this->getUserId() == User::ANONYMOUS_USER) {
            return null;
        }

        return User::find($this->getUserId());
    }

    public function getUserOrString()
    {
        $user = $this->getUser();
        if ($user == null) {
            return $this->getUserString();
        } else {
            return $user;
        }
    }
}
