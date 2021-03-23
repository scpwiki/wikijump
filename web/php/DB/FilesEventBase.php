<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table files_event.
 */
class FilesEventBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='files_event';
        $this->peerName = 'DB\\FilesEventPeer';
        $this->primaryKeyName = 'file_event_id';
        $this->fieldNames = array( 'file_event_id' ,  'filename' ,  'date' ,  'user_id' ,  'user_string' ,  'action' ,  'action_extra' );

        //$this->fieldDefaultValues=
    }






    public function getFileEventId()
    {
        return $this->getFieldValue('file_event_id');
    }

    public function setFileEventId($v1, $raw = false)
    {
        $this->setFieldValue('file_event_id', $v1, $raw);
    }


    public function getFilename()
    {
        return $this->getFieldValue('filename');
    }

    public function setFilename($v1, $raw = false)
    {
        $this->setFieldValue('filename', $v1, $raw);
    }


    public function getDate()
    {
        return $this->getFieldValue('date');
    }

    public function setDate($v1, $raw = false)
    {
        $this->setFieldValue('date', $v1, $raw);
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


    public function getAction()
    {
        return $this->getFieldValue('action');
    }

    public function setAction($v1, $raw = false)
    {
        $this->setFieldValue('action', $v1, $raw);
    }


    public function getActionExtra()
    {
        return $this->getFieldValue('action_extra');
    }

    public function setActionExtra($v1, $raw = false)
    {
        $this->setFieldValue('action_extra', $v1, $raw);
    }
}
