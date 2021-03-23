<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table user_karma.
 */
class UserKarmaPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='user_karma';
        $this->objectName='DB\\UserKarma';
        $this->primaryKeyName = 'user_id';
        $this->fieldNames = array( 'user_id' ,  'points' ,  'level' );
        $this->fieldTypes = array( 'user_id' => 'int',  'points' => 'int',  'level' => 'int');
        $this->defaultValues = array( 'points' => '0',  'level' => '0');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\UserKarmaPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
