<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table global_user_block.
 */
class GlobalUserBlockPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='global_user_block';
        $this->objectName='DB\\GlobalUserBlock';
        $this->primaryKeyName = 'block_id';
        $this->fieldNames = array( 'block_id' ,  'site_id' ,  'user_id' ,  'reason' ,  'date_blocked' );
        $this->fieldTypes = array( 'block_id' => 'serial',  'site_id' => 'int',  'user_id' => 'int',  'reason' => 'text',  'date_blocked' => 'timestamp');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\GlobalUserBlockPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
