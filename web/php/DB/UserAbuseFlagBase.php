<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table user_abuse_flag.
 */
class UserAbuseFlagBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='user_abuse_flag';
        $this->peerName = 'Wikidot\\DB\\UserAbuseFlagPeer';
        $this->primaryKeyName = 'flag_id';
        $this->fieldNames = array( 'flag_id' ,  'user_id' ,  'target_user_id' ,  'site_id' ,  'site_valid' ,  'global_valid' );

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


    public function getTargetUserId()
    {
        return $this->getFieldValue('target_user_id');
    }

    public function setTargetUserId($v1, $raw = false)
    {
        $this->setFieldValue('target_user_id', $v1, $raw);
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
