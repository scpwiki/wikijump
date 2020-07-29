<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table global_ip_block.
 */
class GlobalIpBlockPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='global_ip_block';
        $this->objectName='DB\\GlobalIpBlock';
        $this->primaryKeyName = 'block_id';
        $this->fieldNames = array( 'block_id' ,  'address' ,  'flag_proxy' ,  'reason' ,  'flag_total' ,  'date_blocked' );
        $this->fieldTypes = array( 'block_id' => 'serial',  'address' => 'inet',  'flag_proxy' => 'boolean',  'reason' => 'text',  'flag_total' => 'boolean',  'date_blocked' => 'timestamp');
        $this->defaultValues = array( 'flag_proxy' => 'false',  'flag_total' => 'false');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\GlobalIpBlockPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
