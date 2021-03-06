<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table ozone_user_permission_modifier.
 */
class OzoneUserPermissionModifierPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='ozone_user_permission_modifier';
        $this->objectName='DB\\OzoneUserPermissionModifier';
        $this->primaryKeyName = 'user_permission_id';
        $this->fieldNames = array( 'user_permission_id' ,  'user_id' ,  'permission_id' ,  'modifier' );
        $this->fieldTypes = array( 'user_permission_id' => 'serial',  'user_id' => 'int',  'permission_id' => 'varchar(20)',  'modifier' => 'int');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\OzoneUserPermissionModifierPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
