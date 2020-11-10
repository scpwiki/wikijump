<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table page_abuse_flag.
 */
class PageAbuseFlagBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='page_abuse_flag';
        $this->peerName = 'DB\\PageAbuseFlagPeer';
        $this->primaryKeyName = 'flag_id';
        $this->fieldNames = array( 'flag_id' ,  'user_id' ,  'site_id' ,  'path' ,  'site_valid' ,  'global_valid' );

        //$this->fieldDefaultValues=
    }






    public function getFlagId()
    {
        return $this->getFieldValue('flag_id');
    }

    public function setFlagId($v1, $raw = false)
    {
        $this->setFieldValue('flag_id', $v1, $raw);
    }


    public function getUserId()
    {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw = false)
    {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getPath()
    {
        return $this->getFieldValue('path');
    }

    public function setPath($v1, $raw = false)
    {
        $this->setFieldValue('path', $v1, $raw);
    }


    public function getSiteValid()
    {
        return $this->getFieldValue('site_valid');
    }

    public function setSiteValid($v1, $raw = false)
    {
        $this->setFieldValue('site_valid', $v1, $raw);
    }


    public function getGlobalValid()
    {
        return $this->getFieldValue('global_valid');
    }

    public function setGlobalValid($v1, $raw = false)
    {
        $this->setFieldValue('global_valid', $v1, $raw);
    }
}
