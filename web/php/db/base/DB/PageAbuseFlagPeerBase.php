<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table page_abuse_flag.
 */
class PageAbuseFlagPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='page_abuse_flag';
        $this->objectName='DB\\PageAbuseFlag';
        $this->primaryKeyName = 'flag_id';
        $this->fieldNames = array( 'flag_id' ,  'user_id' ,  'site_id' ,  'path' ,  'site_valid' ,  'global_valid' );
        $this->fieldTypes = array( 'flag_id' => 'serial',  'user_id' => 'int',  'site_id' => 'int',  'path' => 'varchar(100)',  'site_valid' => 'boolean',  'global_valid' => 'boolean');
        $this->defaultValues = array( 'site_valid' => 'true',  'global_valid' => 'true');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\PageAbuseFlagPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
