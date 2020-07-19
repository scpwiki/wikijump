<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table simpletodo_list.
 */
class SimpletodoListPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='simpletodo_list';
        $this->objectName='DB\\SimpletodoList';
        $this->primaryKeyName = 'list_id';
        $this->fieldNames = array( 'list_id' ,  'site_id' ,  'label' ,  'title' ,  'data' );
        $this->fieldTypes = array( 'list_id' => 'serial',  'site_id' => 'int',  'label' => 'varchar(256)',  'title' => 'varchar(256)',  'data' => 'text');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\SimpletodoListPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
