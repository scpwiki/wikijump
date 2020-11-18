<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table private_message.
 */
class PrivateMessagePeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='private_message';
        $this->objectName='DB\\PrivateMessage';
        $this->primaryKeyName = 'message_id';
        $this->fieldNames = array( 'message_id' ,  'from_user_id' ,  'to_user_id' ,  'subject' ,  'body' ,  'date' ,  'flag' ,  'flag_new' );
        $this->fieldTypes = array( 'message_id' => 'serial',  'from_user_id' => 'int',  'to_user_id' => 'int',  'subject' => 'varchar(256)',  'body' => 'text',  'date' => 'timestamp',  'flag' => 'int',  'flag_new' => 'boolean');
        $this->defaultValues = array( 'flag' => '0',  'flag_new' => 'true');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\PrivateMessagePeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
