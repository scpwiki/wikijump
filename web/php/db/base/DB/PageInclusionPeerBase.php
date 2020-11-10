<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table page_inclusion.
 */
class PageInclusionPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='page_inclusion';
        $this->objectName='DB\\PageInclusion';
        $this->primaryKeyName = 'inclusion_id';
        $this->fieldNames = array( 'inclusion_id' ,  'site_id' ,  'including_page_id' ,  'included_page_id' ,  'included_page_name' );
        $this->fieldTypes = array( 'inclusion_id' => 'serial',  'site_id' => 'int',  'including_page_id' => 'int',  'included_page_id' => 'int',  'included_page_name' => 'varchar(128)');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\PageInclusionPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
