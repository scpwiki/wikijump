<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table page_link.
 */
class PageLinkPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='page_link';
        $this->objectName='Wikidot\\DB\\PageLink';
        $this->primaryKeyName = 'link_id';
        $this->fieldNames = array( 'link_id' ,  'site_id' ,  'from_page_id' ,  'to_page_id' ,  'to_page_name' );
        $this->fieldTypes = array( 'link_id' => 'serial',  'site_id' => 'int',  'from_page_id' => 'int',  'to_page_id' => 'int',  'to_page_name' => 'varchar(128)');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\PageLinkPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
