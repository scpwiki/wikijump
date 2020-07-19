<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table private_user_block.
 */
class PrivateUserBlockPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='private_user_block';
        $this->objectName='DB\\PrivateUserBlock';
        $this->primaryKeyName = 'block_id';
        $this->fieldNames = array( 'block_id' ,  'user_id' ,  'blocked_user_id' );
        $this->fieldTypes = array( 'block_id' => 'serial',  'user_id' => 'int',  'blocked_user_id' => 'int');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\PrivateUserBlockPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
