<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table ozone_group_permission_modifier.
 */
class OzoneGroupPermissionModifierPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='ozone_group_permission_modifier';
        $this->objectName='Wikidot\\DB\\OzoneGroupPermissionModifier';
        $this->primaryKeyName = 'group_permission_id';
        $this->fieldNames = array( 'group_permission_id' ,  'group_id' ,  'permission_id' ,  'modifier' );
        $this->fieldTypes = array( 'group_permission_id' => 'serial',  'group_id' => 'varchar(20)',  'permission_id' => 'varchar(20)',  'modifier' => 'int');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\OzoneGroupPermissionModifierPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
