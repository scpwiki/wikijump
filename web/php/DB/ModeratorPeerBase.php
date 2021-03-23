<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table moderator.
 */
class ModeratorPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='moderator';
        $this->objectName='Wikidot\\DB\\Moderator';
        $this->primaryKeyName = 'moderator_id';
        $this->fieldNames = array( 'moderator_id' ,  'site_id' ,  'user_id' ,  'permissions' );
        $this->fieldTypes = array( 'moderator_id' => 'serial',  'site_id' => 'int',  'user_id' => 'int',  'permissions' => 'char(10)');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\ModeratorPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
