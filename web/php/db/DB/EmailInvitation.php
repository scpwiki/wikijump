<?php
namespace DB;

/**
 * Object Model class.
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
