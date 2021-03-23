<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table license.
 */
class LicensePeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='license';
        $this->objectName='Wikidot\\DB\\License';
        $this->primaryKeyName = 'license_id';
        $this->fieldNames = array( 'license_id' ,  'name' ,  'description' ,  'sort' );
        $this->fieldTypes = array( 'license_id' => 'serial',  'name' => 'varchar(100)',  'description' => 'text',  'sort' => 'int');
        $this->defaultValues = array( 'sort' => '0');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\LicensePeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
