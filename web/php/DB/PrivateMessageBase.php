<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table private_message.
 */
class PrivateMessageBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='private_message';
        $this->peerName = 'Wikidot\\DB\\PrivateMessagePeer';
        $this->primaryKeyName = 'message_id';
        $this->fieldNames = array( 'message_id' ,  'from_user_id' ,  'to_user_id' ,  'subject' ,  'body' ,  'date' ,  'flag' ,  'flag_new' );

        //$this->fieldDefaultValues=
    }






    public function getMessageId()
    {
        return $this->getFieldValue('message_id');
    }

    public function setMessageId($v1, $raw = false)
    {
        $this->setFieldValue('message_id', $v1, $raw);
    }


    public function getFromUserId()
    {
        return $this->getFieldValue('from_user_id');
    }

    public function setFromUserId($v1, $raw = false)
    {
        $this->setFieldValue('from_user_id', $v1, $raw);
    }


    public function getToUserId()
    {
        return $this->getFieldValue('to_user_id');
    }

    public function setToUserId($v1, $raw = false)
    {
        $this->setFieldValue('to_user_id', $v1, $raw);
    }


    public function getSubject()
    {
        return $this->getFieldValue('subject');
    }

    public function setSubject($v1, $raw = false)
    {
        $this->setFieldValue('subject', $v1, $raw);
    }


    public function getBody()
    {
        return $this->getFieldValue('body');
    }

    public function setBody($v1, $raw = false)
    {
        $this->setFieldValue('body', $v1, $raw);
    }


    public function getDate()
    {
        return $this->getFieldValue('date');
    }

    public function setDate($v1, $raw = false)
    {
        $this->setFieldValue('date', $v1, $raw);
    }


    public function getFlag()
    {
        return $this->getFieldValue('flag');
    }

    public function setFlag($v1, $raw = false)
    {
        $this->setFieldValue('flag', $v1, $raw);
    }


    public function getFlagNew()
    {
        return $this->getFieldValue('flag_new');
    }

    public function setFlagNew($v1, $raw = false)
    {
        $this->setFieldValue('flag_new', $v1, $raw);
    }
}
