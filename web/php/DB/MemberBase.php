<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table member.
 */
class MemberBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='member';
        $this->peerName = 'Wikidot\\DB\\MemberPeer';
        $this->primaryKeyName = 'member_id';
        $this->fieldNames = array( 'member_id' ,  'site_id' ,  'user_id' ,  'date_joined' ,  'allow_newsletter' );

        //$this->fieldDefaultValues=
    }






    public function getMemberId()
    {
        return $this->getFieldValue('member_id');
    }

    public function setMemberId($v1, $raw = false)
    {
        $this->setFieldValue('member_id', $v1, $raw);
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


    public function getDateJoined()
    {
        return $this->getFieldValue('date_joined');
    }

    public function setDateJoined($v1, $raw = false)
    {
        $this->setFieldValue('date_joined', $v1, $raw);
    }


    public function getAllowNewsletter()
    {
        return $this->getFieldValue('allow_newsletter');
    }

    public function setAllowNewsletter($v1, $raw = false)
    {
        $this->setFieldValue('allow_newsletter', $v1, $raw);
    }
}
