<?php

namespace Wikidot\DB;




use Illuminate\Support\Facades\Auth;
use Ozone\Framework\Database\BaseDBObject;
use Ozone\Framework\Database\Criteria;
use Wikijump\Models\User;

/**
 * Base Class mapped to the database table ozone_session.
 */
class OzoneSessionBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='ozone_session';
        $this->peerName = 'Wikidot\\DB\\OzoneSessionPeer';
        $this->primaryKeyName = 'session_id';
        $this->fieldNames = array( 'session_id' ,  'started' ,  'last_accessed' ,  'ip_address' ,  'ip_address_ssl' ,  'ua_hash' ,  'check_ip' ,  'infinite' ,  'user_id' ,  'serialized_datablock' );

        //$this->fieldDefaultValues=
    }



    public function getOzoneUser()
    {
        return Auth::user();
    }

    public function setOzoneUser($primaryObject)
    {
        $this->fieldValues['user_id'] = $primaryObject->getFieldValue('user_id');
    }



    public function getSessionId()
    {
        return $this->getFieldValue('session_id');
    }

    public function setSessionId($v1, $raw = false)
    {
        $this->setFieldValue('session_id', $v1, $raw);
    }


    public function getStarted()
    {
        return $this->getFieldValue('started');
    }

    public function setStarted($v1, $raw = false)
    {
        $this->setFieldValue('started', $v1, $raw);
    }


    public function getLastAccessed()
    {
        return $this->getFieldValue('last_accessed');
    }

    public function setLastAccessed($v1, $raw = false)
    {
        $this->setFieldValue('last_accessed', $v1, $raw);
    }


    public function getIpAddress()
    {
        return $this->getFieldValue('ip_address');
    }

    public function setIpAddress($v1, $raw = false)
    {
        $this->setFieldValue('ip_address', $v1, $raw);
    }


    public function getIpAddressSsl()
    {
        return $this->getFieldValue('ip_address_ssl');
    }

    public function setIpAddressSsl($v1, $raw = false)
    {
        $this->setFieldValue('ip_address_ssl', $v1, $raw);
    }


    public function getUaHash()
    {
        return $this->getFieldValue('ua_hash');
    }

    public function setUaHash($v1, $raw = false)
    {
        $this->setFieldValue('ua_hash', $v1, $raw);
    }


    public function getCheckIp()
    {
        return $this->getFieldValue('check_ip');
    }

    public function setCheckIp($v1, $raw = false)
    {
        $this->setFieldValue('check_ip', $v1, $raw);
    }


    public function getInfinite()
    {
        return $this->getFieldValue('infinite');
    }

    public function setInfinite($v1, $raw = false)
    {
        $this->setFieldValue('infinite', $v1, $raw);
    }


    public function getUserId()
    {
            return Auth::id();
    }

    public function setUserId($v1, $raw = false)
    {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getSerializedDatablock()
    {
        return $this->getFieldValue('serialized_datablock');
    }

    public function setSerializedDatablock($v1, $raw = false)
    {
        $this->setFieldValue('serialized_datablock', $v1, $raw);
    }
}
