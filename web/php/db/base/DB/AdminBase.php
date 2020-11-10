<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table admin.
 */
class AdminBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='admin';
        $this->peerName = 'DB\\AdminPeer';
        $this->primaryKeyName = 'admin_id';
        $this->fieldNames = array( 'admin_id' ,  'site_id' ,  'user_id' ,  'founder' );

        //$this->fieldDefaultValues=
    }






    public function getAdminId()
    {
        return $this->getFieldValue('admin_id');
    }

    public function setAdminId($v1, $raw = false)
    {
        $this->setFieldValue('admin_id', $v1, $raw);
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


    public function getFounder()
    {
        return $this->getFieldValue('founder');
    }

    public function setFounder($v1, $raw = false)
    {
        $this->setFieldValue('founder', $v1, $raw);
    }
}
