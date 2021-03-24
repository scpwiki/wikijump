<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table forum_group.
 */
class ForumGroupPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='forum_group';
        $this->objectName='Wikidot\\DB\\ForumGroup';
        $this->primaryKeyName = 'group_id';
        $this->fieldNames = array( 'group_id' ,  'name' ,  'description' ,  'sort_index' ,  'site_id' ,  'visible' );
        $this->fieldTypes = array( 'group_id' => 'serial',  'name' => 'varchar(80)',  'description' => 'text',  'sort_index' => 'int',  'site_id' => 'int',  'visible' => 'boolean');
        $this->defaultValues = array( 'sort_index' => '0',  'visible' => 'true');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\ForumGroupPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
