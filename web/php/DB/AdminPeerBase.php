<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table admin.
 */
class AdminPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='admin';
        $this->objectName='DB\\Admin';
        $this->primaryKeyName = 'admin_id';
        $this->fieldNames = array( 'admin_id' ,  'site_id' ,  'user_id' ,  'founder' );
        $this->fieldTypes = array( 'admin_id' => 'serial',  'site_id' => 'int',  'user_id' => 'int',  'founder' => 'boolean');
        $this->defaultValues = array( 'founder' => 'false');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\AdminPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
