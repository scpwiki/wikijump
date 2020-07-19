<?php
namespace DB;

/**
 * Object Model class.
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
