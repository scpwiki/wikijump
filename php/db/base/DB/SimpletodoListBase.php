<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table simpletodo_list.
 */
class SimpletodoListBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='simpletodo_list';
        $this->peerName = 'DB\\SimpletodoListPeer';
        $this->primaryKeyName = 'list_id';
        $this->fieldNames = array( 'list_id' ,  'site_id' ,  'label' ,  'title' ,  'data' );

        //$this->fieldDefaultValues=
    }






    public function getListId()
    {
        return $this->getFieldValue('list_id');
    }

    public function setListId($v1, $raw = false)
    {
        $this->setFieldValue('list_id', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getLabel()
    {
        return $this->getFieldValue('label');
    }

    public function setLabel($v1, $raw = false)
    {
        $this->setFieldValue('label', $v1, $raw);
    }


    public function getTitle()
    {
        return $this->getFieldValue('title');
    }

    public function setTitle($v1, $raw = false)
    {
        $this->setFieldValue('title', $v1, $raw);
    }


    public function getData()
    {
        return $this->getFieldValue('data');
    }

    public function setData($v1, $raw = false)
    {
        $this->setFieldValue('data', $v1, $raw);
    }
}
