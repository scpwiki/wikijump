<?php

namespace Wikidot\DB;


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
        return OzoneUserPeer::instance()->selectByPrimaryKey($this->getByUserId());
    }
}
