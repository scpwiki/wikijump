<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table page_source.
 */
class PageSourcePeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='page_source';
        $this->objectName='DB\\PageSource';
        $this->primaryKeyName = 'source_id';
        $this->fieldNames = array( 'source_id' ,  'text' );
        $this->fieldTypes = array( 'source_id' => 'serial',  'text' => 'text');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\PageSourcePeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
