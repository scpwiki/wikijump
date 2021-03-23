<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table ucookie.
 */
class UcookiePeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='ucookie';
        $this->objectName='DB\\Ucookie';
        $this->primaryKeyName = 'ucookie_id';
        $this->fieldNames = array( 'ucookie_id' ,  'site_id' ,  'session_id' ,  'date_granted' );
        $this->fieldTypes = array( 'ucookie_id' => 'varchar(100)',  'site_id' => 'int',  'session_id' => 'varchar(60)',  'date_granted' => 'timestamp');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\UcookiePeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
