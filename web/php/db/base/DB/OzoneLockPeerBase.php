<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table ozone_lock.
 */
class OzoneLockPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='ozone_lock';
        $this->objectName='DB\\OzoneLock';
        $this->primaryKeyName = 'key';
        $this->fieldNames = array( 'key' );
        $this->fieldTypes = array( 'key' => 'varchar(100)');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\OzoneLockPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
