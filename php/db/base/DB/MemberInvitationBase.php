<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table member_invitation.
 */
class MemberInvitationBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='member_invitation';
        $this->peerName = 'DB\\MemberInvitationPeer';
        $this->primaryKeyName = 'invitation_id';
        $this->fieldNames = array( 'invitation_id' ,  'site_id' ,  'user_id' ,  'by_user_id' ,  'date' ,  'body' );

        //$this->fieldDefaultValues=
    }






    public function getInvitationId()
    {
        return $this->getFieldValue('invitation_id');
    }

    public function setInvitationId($v1, $raw = false)
    {
        $this->setFieldValue('invitation_id', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getUserId()
    {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw = false)
    {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getByUserId()
    {
        return $this->getFieldValue('by_user_id');
    }

    public function setByUserId($v1, $raw = false)
    {
        $this->setFieldValue('by_user_id', $v1, $raw);
    }


    public function getDate()
    {
        return $this->getFieldValue('date');
    }

    public function setDate($v1, $raw = false)
    {
        $this->setFieldValue('date', $v1, $raw);
    }


    public function getBody()
    {
        return $this->getFieldValue('body');
    }

    public function setBody($v1, $raw = false)
    {
        $this->setFieldValue('body', $v1, $raw);
    }
}
