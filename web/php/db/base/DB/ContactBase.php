<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table contact.
 */
class ContactBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='contact';
        $this->peerName = 'DB\\ContactPeer';
        $this->primaryKeyName = 'contact_id';
        $this->fieldNames = array( 'contact_id' ,  'user_id' ,  'target_user_id' );

        //$this->fieldDefaultValues=
    }






    public function getContactId()
    {
        return $this->getFieldValue('contact_id');
    }

    public function setContactId($v1, $raw = false)
    {
        $this->setFieldValue('contact_id', $v1, $raw);
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
}
