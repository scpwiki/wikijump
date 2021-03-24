<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table ozone_group.
 */
class OzoneGroupPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='ozone_group';
        $this->objectName='Wikidot\\DB\\OzoneGroup';
        $this->primaryKeyName = 'group_id';
        $this->fieldNames = array( 'group_id' ,  'parent_group_id' ,  'name' ,  'description' );
        $this->fieldTypes = array( 'group_id' => 'serial',  'parent_group_id' => 'int',  'name' => 'varchar(50)',  'description' => 'text');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\OzoneGroupPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
