<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table fts_entry.
 */
class FtsEntryBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='fts_entry';
        $this->peerName = 'DB\\FtsEntryPeer';
        $this->primaryKeyName = 'fts_id';
        $this->fieldNames = array( 'fts_id' ,  'page_id' ,  'title' ,  'unix_name' ,  'thread_id' ,  'site_id' ,  'text' ,  'vector' );

        //$this->fieldDefaultValues=
    }






    public function getFtsId()
    {
        return $this->getFieldValue('fts_id');
    }

    public function setFtsId($v1, $raw = false)
    {
        $this->setFieldValue('fts_id', $v1, $raw);
    }


    public function getPageId()
    {
        return $this->getFieldValue('page_id');
    }

    public function setPageId($v1, $raw = false)
    {
        $this->setFieldValue('page_id', $v1, $raw);
    }


    public function getTitle()
    {
        return $this->getFieldValue('title');
    }

    public function setTitle($v1, $raw = false)
    {
        $this->setFieldValue('title', $v1, $raw);
    }


    public function getUnixName()
    {
        return $this->getFieldValue('unix_name');
    }

    public function setUnixName($v1, $raw = false)
    {
        $this->setFieldValue('unix_name', $v1, $raw);
    }


    public function getThreadId()
    {
        return $this->getFieldValue('thread_id');
    }

    public function setThreadId($v1, $raw = false)
    {
        $this->setFieldValue('thread_id', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getText()
    {
        return $this->getFieldValue('text');
    }

    public function setText($v1, $raw = false)
    {
        $this->setFieldValue('text', $v1, $raw);
    }


    public function getVector()
    {
        return $this->getFieldValue('vector');
    }

    public function setVector($v1, $raw = false)
    {
        $this->setFieldValue('vector', $v1, $raw);
    }
}
