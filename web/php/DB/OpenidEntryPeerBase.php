<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table openid_entry.
 */
class OpenidEntryPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='openid_entry';
        $this->objectName='Wikidot\\DB\\OpenidEntry';
        $this->primaryKeyName = 'openid_id';
        $this->fieldNames = array( 'openid_id' ,  'site_id' ,  'page_id' ,  'type' ,  'user_id' ,  'url' ,  'server_url' );
        $this->fieldTypes = array( 'openid_id' => 'serial',  'site_id' => 'int',  'page_id' => 'int',  'type' => 'varchar(10)',  'user_id' => 'int',  'url' => 'varchar(100)',  'server_url' => 'varchar(100)');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\OpenidEntryPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
