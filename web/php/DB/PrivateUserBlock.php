<?php

namespace Wikidot\DB;


/**
 * Object Model Class.
 *
 */
class PrivateUserBlock extends PrivateUserBlockBase
{

    public function getBlockedUser()
    {
        return OzoneUserPeer::instance()->selectByPrimaryKey($this->getBlockedUserId());
    }
}
