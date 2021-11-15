<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table page_edit_lock.
 */
class PageEditLockBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='page_edit_lock';
        $this->peerName = 'Wikidot\\DB\\PageEditLockPeer';
        $this->primaryKeyName = 'lock_id';
        $this->fieldNames = array( 'lock_id' ,  'page_id' ,  'mode' ,   'page_unix_name' ,  'site_id' ,  'user_id' ,  'user_string' ,  'session_id' ,  'date_started' ,  'date_last_accessed' ,  'secret' );

        //$this->fieldDefaultValues=
    }






    public function getLockId()
    {
        return $this->getFieldValue('lock_id');
    }

    public function setLockId($v1, $raw = false)
    {
        $this->setFieldValue('lock_id', $v1, $raw);
    }


    public function getPageId()
    {
        return $this->getFieldValue('page_id');
    }

    public function setPageId($v1, $raw = false)
    {
        $this->setFieldValue('page_id', $v1, $raw);
    }


    public function getMode()
    {
        return $this->getFieldValue('mode');
    }

    public function setMode($v1, $raw = false)
    {
        $this->setFieldValue('mode', $v1, $raw);
    }


    public function getPageUnixName()
    {
        return $this->getFieldValue('page_unix_name');
    }

    public function setPageUnixName($v1, $raw = false)
    {
        $this->setFieldValue('page_unix_name', $v1, $raw);
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


    public function getUserString()
    {
        return $this->getFieldValue('user_string');
    }

    public function setUserString($v1, $raw = false)
    {
        $this->setFieldValue('user_string', $v1, $raw);
    }


    public function getSessionId()
    {
        return $this->getFieldValue('session_id');
    }

    public function setSessionId($v1, $raw = false)
    {
        $this->setFieldValue('session_id', $v1, $raw);
    }


    public function getDateStarted()
    {
        return $this->getFieldValue('date_started');
    }

    public function setDateStarted($v1, $raw = false)
    {
        $this->setFieldValue('date_started', $v1, $raw);
    }


    public function getDateLastAccessed()
    {
        return $this->getFieldValue('date_last_accessed');
    }

    public function setDateLastAccessed($v1, $raw = false)
    {
        $this->setFieldValue('date_last_accessed', $v1, $raw);
    }


    public function getSecret()
    {
        return $this->getFieldValue('secret');
    }

    public function setSecret($v1, $raw = false)
    {
        $this->setFieldValue('secret', $v1, $raw);
    }
}
