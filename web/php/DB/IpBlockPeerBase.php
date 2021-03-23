<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table ip_block.
 */
class IpBlockPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='ip_block';
        $this->objectName='DB\\IpBlock';
        $this->primaryKeyName = 'block_id';
        $this->fieldNames = array( 'block_id' ,  'site_id' ,  'ip' ,  'flag_proxy' ,  'reason' ,  'date_blocked' );
        $this->fieldTypes = array( 'block_id' => 'serial',  'site_id' => 'int',  'ip' => 'inet',  'flag_proxy' => 'boolean',  'reason' => 'text',  'date_blocked' => 'timestamp');
        $this->defaultValues = array( 'flag_proxy' => 'false');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\IpBlockPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
