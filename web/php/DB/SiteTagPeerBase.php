<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table site_tag.
 */
class SiteTagPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='site_tag';
        $this->objectName='Wikidot\\DB\\SiteTag';
        $this->primaryKeyName = 'tag_id';
        $this->fieldNames = array( 'tag_id' ,  'site_id' ,  'tag' );
        $this->fieldTypes = array( 'tag_id' => 'serial',  'site_id' => 'int',  'tag' => 'varchar(64)');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\SiteTagPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
