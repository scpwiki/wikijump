<?php

namespace Wikidot\DB;


/**
 * Object Model Class.
 *
 */
class EmailInvitation extends EmailInvitationBase
{

    public function getUser()
    {
        $user = OzoneUserPeer::instance()->selectByPrimaryKey($this->getUserId());
        return $user;
    }
}
