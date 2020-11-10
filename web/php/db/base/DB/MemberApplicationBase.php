<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table member_application.
 */
class MemberApplicationBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='member_application';
        $this->peerName = 'DB\\MemberApplicationPeer';
        $this->primaryKeyName = 'application_id';
        $this->fieldNames = array( 'application_id' ,  'site_id' ,  'user_id' ,  'status' ,  'date' ,  'comment' ,  'reply' );

        //$this->fieldDefaultValues=
    }






    public function getApplicationId()
    {
        return $this->getFieldValue('application_id');
    }

    public function setApplicationId($v1, $raw = false)
    {
        $this->setFieldValue('application_id', $v1, $raw);
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


    public function getStatus()
    {
        return $this->getFieldValue('status');
    }

    public function setStatus($v1, $raw = false)
    {
        $this->setFieldValue('status', $v1, $raw);
    }


    public function getDate()
    {
        return $this->getFieldValue('date');
    }

    public function setDate($v1, $raw = false)
    {
        $this->setFieldValue('date', $v1, $raw);
    }


    public function getComment()
    {
        return $this->getFieldValue('comment');
    }

    public function setComment($v1, $raw = false)
    {
        $this->setFieldValue('comment', $v1, $raw);
    }


    public function getReply()
    {
        return $this->getFieldValue('reply');
    }

    public function setReply($v1, $raw = false)
    {
        $this->setFieldValue('reply', $v1, $raw);
    }
}
