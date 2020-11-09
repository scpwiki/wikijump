<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table file.
 */
class FileBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='file';
        $this->peerName = 'DB\\FilePeer';
        $this->primaryKeyName = 'file_id';
        $this->fieldNames = array( 'file_id' ,  'page_id' ,  'site_id' ,  'filename' ,  'mimetype' ,  'description' ,  'description_short' ,  'comment' ,  'size' ,  'date_added' ,  'user_id' ,  'user_string' ,  'has_resized' );

        //$this->fieldDefaultValues=
    }






    public function getFileId()
    {
        return $this->getFieldValue('file_id');
    }

    public function setFileId($v1, $raw = false)
    {
        $this->setFieldValue('file_id', $v1, $raw);
    }


    public function getPageId()
    {
        return $this->getFieldValue('page_id');
    }

    public function setPageId($v1, $raw = false)
    {
        $this->setFieldValue('page_id', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getFilename()
    {
        return $this->getFieldValue('filename');
    }

    public function setFilename($v1, $raw = false)
    {
        $this->setFieldValue('filename', $v1, $raw);
    }


    public function getMimetype()
    {
        return $this->getFieldValue('mimetype');
    }

    public function setMimetype($v1, $raw = false)
    {
        $this->setFieldValue('mimetype', $v1, $raw);
    }


    public function getDescription()
    {
        return $this->getFieldValue('description');
    }

    public function setDescription($v1, $raw = false)
    {
        $this->setFieldValue('description', $v1, $raw);
    }


    public function getDescriptionShort()
    {
        return $this->getFieldValue('description_short');
    }

    public function setDescriptionShort($v1, $raw = false)
    {
        $this->setFieldValue('description_short', $v1, $raw);
    }


    public function getComment()
    {
        return $this->getFieldValue('comment');
    }

    public function setComment($v1, $raw = false)
    {
        $this->setFieldValue('comment', $v1, $raw);
    }


    public function getSize()
    {
        return $this->getFieldValue('size');
    }

    public function setSize($v1, $raw = false)
    {
        $this->setFieldValue('size', $v1, $raw);
    }


    public function getDateAdded()
    {
        return $this->getFieldValue('date_added');
    }

    public function setDateAdded($v1, $raw = false)
    {
        $this->setFieldValue('date_added', $v1, $raw);
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


    public function getHasResized()
    {
        return $this->getFieldValue('has_resized');
    }

    public function setHasResized($v1, $raw = false)
    {
        $this->setFieldValue('has_resized', $v1, $raw);
    }
}
