<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table page_tag.
 */
class PageTagPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='page_tag';
        $this->objectName='DB\\PageTag';
        $this->primaryKeyName = 'tag_id';
        $this->fieldNames = array( 'tag_id' ,  'site_id' ,  'page_id' ,  'tag' );
        $this->fieldTypes = array( 'tag_id' => 'bigserial',  'site_id' => 'int',  'page_id' => 'int',  'tag' => 'varchar(64)');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            self::$peerInstance = new PageTagPeer();
        }
        return self::$peerInstance;
    }
}
