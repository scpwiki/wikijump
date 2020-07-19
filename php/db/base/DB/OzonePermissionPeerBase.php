<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table ozone_permission.
 */
class OzonePermissionPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='ozone_permission';
        $this->objectName='DB\\OzonePermission';
        $this->primaryKeyName = 'permission_id';
        $this->fieldNames = array( 'permission_id' ,  'name' ,  'description' );
        $this->fieldTypes = array( 'permission_id' => 'serial',  'name' => 'varchar(50)',  'description' => 'text');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\OzonePermissionPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
