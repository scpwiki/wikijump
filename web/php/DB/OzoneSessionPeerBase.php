<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table ozone_session.
 */
class OzoneSessionPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='ozone_session';
        $this->objectName='Wikidot\\DB\\OzoneSession';
        $this->primaryKeyName = 'session_id';
        $this->fieldNames = array( 'session_id' ,  'started' ,  'last_accessed' ,  'ip_address' ,  'ip_address_ssl' ,  'ua_hash' ,  'check_ip' ,  'infinite' ,  'user_id' ,  'serialized_datablock' );
        $this->fieldTypes = array( 'session_id' => 'varchar(60)',  'started' => 'timestamp',  'last_accessed' => 'timestamp',  'ip_address' => 'varchar(90)',  'ip_address_ssl' => 'varchar(90)',  'ua_hash' => 'varchar(256)',  'check_ip' => 'boolean',  'infinite' => 'boolean',  'user_id' => 'int',  'serialized_datablock' => 'bytea');
        $this->defaultValues = array( 'check_ip' => 'false',  'infinite' => 'false');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\OzoneSessionPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
