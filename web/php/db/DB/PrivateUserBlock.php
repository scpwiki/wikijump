<?php
namespace DB;

/**
 * Object Model class.
 *
 */
class PrivateUserBlock extends PrivateUserBlockBase
{

    public function getBlockedUser()
    {
        return OzoneUserPeer::instance()->selectByPrimaryKey($this->getBlockedUserId());
    }
}
