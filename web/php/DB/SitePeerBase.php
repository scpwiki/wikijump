<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table site.
 */
class SitePeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='site';
        $this->objectName='Wikidot\\DB\\Site';
        $this->primaryKeyName = 'site_id';
        $this->fieldNames = array( 'site_id' ,  'name' ,  'subtitle' ,  'slug' ,  'description' ,  'language' ,  'date_created' ,  'custom_domain' ,  'default_page' ,  'visible' ,  'private' ,  'deleted' );
        $this->fieldTypes = array( 'site_id' => 'serial',  'name' => 'varchar(100)',  'subtitle' => 'varchar(60)',  'slug' => 'text',  'description' => 'text',  'language' => 'varchar(10)',  'date_created' => 'timestamp',  'custom_domain' => 'varchar(60)',  'default_page' => 'varchar(80)',  'visible' => 'boolean',  'private' => 'boolean',  'deleted' => 'boolean');
        $this->defaultValues = array( 'language' => 'en',  'default_page' => 'start',  'visible' => 'true',  'private' => 'false',  'deleted' => 'false');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            self::$peerInstance = new SitePeer();
        }
        return self::$peerInstance;
    }
}
