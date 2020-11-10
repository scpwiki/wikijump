<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table storage_item.
 */
class StorageItemPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='storage_item';
        $this->objectName='DB\\StorageItem';
        $this->primaryKeyName = 'item_id';
        $this->fieldNames = array( 'item_id' ,  'date' ,  'timeout' ,  'data' );
        $this->fieldTypes = array( 'item_id' => 'varchar(256)',  'date' => 'timestamp',  'timeout' => 'int',  'data' => 'bytea');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\StorageItemPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
