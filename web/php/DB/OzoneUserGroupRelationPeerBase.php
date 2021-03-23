<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table ozone_user_group_relation.
 */
class OzoneUserGroupRelationPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='ozone_user_group_relation';
        $this->objectName='Wikidot\\DB\\OzoneUserGroupRelation';
        $this->primaryKeyName = 'user_group_id';
        $this->fieldNames = array( 'user_group_id' ,  'user_id' ,  'group_id' );
        $this->fieldTypes = array( 'user_group_id' => 'serial',  'user_id' => 'int',  'group_id' => 'int');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\OzoneUserGroupRelationPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
