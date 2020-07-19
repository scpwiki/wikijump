<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table site_tag.
 */
class SiteTagPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='site_tag';
        $this->objectName='DB\\SiteTag';
        $this->primaryKeyName = 'tag_id';
        $this->fieldNames = array( 'tag_id' ,  'site_id' ,  'tag' );
        $this->fieldTypes = array( 'tag_id' => 'serial',  'site_id' => 'int',  'tag' => 'varchar(64)');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\SiteTagPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
