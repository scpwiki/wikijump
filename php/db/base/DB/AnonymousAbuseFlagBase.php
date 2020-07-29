<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table anonymous_abuse_flag.
 */
class AnonymousAbuseFlagBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='anonymous_abuse_flag';
        $this->peerName = 'DB\\AnonymousAbuseFlagPeer';
        $this->primaryKeyName = 'flag_id';
        $this->fieldNames = array( 'flag_id' ,  'user_id' ,  'address' ,  'proxy' ,  'site_id' ,  'site_valid' ,  'global_valid' );

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


    public function getAddress()
    {
        return $this->getFieldValue('address');
    }

    public function setAddress($v1, $raw = false)
    {
        $this->setFieldValue('address', $v1, $raw);
    }


    public function getProxy()
    {
        return $this->getFieldValue('proxy');
    }

    public function setProxy($v1, $raw = false)
    {
        $this->setFieldValue('proxy', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
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
