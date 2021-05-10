<?php

namespace Wikidot\DB;


use Wikijump\Models\User;

/**
 * Object Model Class.
 *
 */
class MemberInvitation extends MemberInvitationBase
{

    public function getSite()
    {
        return SitePeer::instance()->selectByPrimaryKey($this->getSiteId());
    }

    public function getByUser()
    {
        return User::find($this->getByUserId());
    }
}
