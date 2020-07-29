<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table storage_item.
 */
class StorageItemBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='storage_item';
        $this->peerName = 'DB\\StorageItemPeer';
        $this->primaryKeyName = 'item_id';
        $this->fieldNames = array( 'item_id' ,  'date' ,  'timeout' ,  'data' );

        //$this->fieldDefaultValues=
    }






    public function getItemId()
    {
        return $this->getFieldValue('item_id');
    }

    public function setItemId($v1, $raw = false)
    {
        $this->setFieldValue('item_id', $v1, $raw);
    }


    public function getDate()
    {
        return $this->getFieldValue('date');
    }

    public function setDate($v1, $raw = false)
    {
        $this->setFieldValue('date', $v1, $raw);
    }


    public function getTimeout()
    {
        return $this->getFieldValue('timeout');
    }

    public function setTimeout($v1, $raw = false)
    {
        $this->setFieldValue('timeout', $v1, $raw);
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
