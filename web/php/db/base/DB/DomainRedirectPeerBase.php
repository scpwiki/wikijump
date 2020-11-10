<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table domain_redirect.
 */
class DomainRedirectPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='domain_redirect';
        $this->objectName='DB\\DomainRedirect';
        $this->primaryKeyName = 'redirect_id';
        $this->fieldNames = array( 'redirect_id' ,  'site_id' ,  'url' );
        $this->fieldTypes = array( 'redirect_id' => 'serial',  'site_id' => 'int',  'url' => 'varchar(80)');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\DomainRedirectPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
