<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table page_external_link.
 */
class PageExternalLinkPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='page_external_link';
        $this->objectName='DB\\PageExternalLink';
        $this->primaryKeyName = 'link_id';
        $this->fieldNames = array( 'link_id' ,  'site_id' ,  'page_id' ,  'to_url' ,  'pinged' ,  'ping_status' ,  'date' );
        $this->fieldTypes = array( 'link_id' => 'serial',  'site_id' => 'int',  'page_id' => 'int',  'to_url' => 'varchar(512)',  'pinged' => 'boolean',  'ping_status' => 'varchar(256)',  'date' => 'timestamp');
        $this->defaultValues = array( 'pinged' => 'false');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\PageExternalLinkPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
