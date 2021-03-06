<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table member_application.
 */
class MemberApplicationPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='member_application';
        $this->objectName='DB\\MemberApplication';
        $this->primaryKeyName = 'application_id';
        $this->fieldNames = array( 'application_id' ,  'site_id' ,  'user_id' ,  'status' ,  'date' ,  'comment' ,  'reply' );
        $this->fieldTypes = array( 'application_id' => 'serial',  'site_id' => 'int',  'user_id' => 'int',  'status' => 'varchar(20)',  'date' => 'timestamp',  'comment' => 'text',  'reply' => 'text');
        $this->defaultValues = array( 'status' => 'pending');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\MemberApplicationPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
