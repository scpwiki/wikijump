<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table ozone_user.
 */
class OzoneUserPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='ozone_user';
        $this->objectName='DB\\OzoneUser';
        $this->primaryKeyName = 'user_id';
        $this->fieldNames = array( 'user_id' ,  'name' ,  'nick_name' ,  'password' ,  'email' ,  'unix_name' ,  'last_login' ,  'registered_date' ,  'super_admin' ,  'super_moderator' ,  'language' );
        $this->fieldTypes = array( 'user_id' => 'serial',  'name' => 'varchar(99)',  'nick_name' => 'varchar(70)',  'password' => 'varchar(255)',  'email' => 'varchar(99)',  'unix_name' => 'varchar(99)',  'last_login' => 'timestamp',  'registered_date' => 'timestamp',  'super_admin' => 'boolean',  'super_moderator' => 'boolean',  'language' => 'varchar(10)');
        $this->defaultValues = array( 'super_admin' => 'false',  'super_moderator' => 'false',  'language' => 'en');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\OzoneUserPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
