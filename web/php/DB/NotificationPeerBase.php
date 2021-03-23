<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table notification.
 */
class NotificationPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='notification';
        $this->objectName='DB\\Notification';
        $this->primaryKeyName = 'notification_id';
        $this->fieldNames = array( 'notification_id' ,  'user_id' ,  'body' ,  'type' ,  'viewed' ,  'date' ,  'Extra' ,  'notify_online' ,  'notify_feed' ,  'notify_email' );
        $this->fieldTypes = array( 'notification_id' => 'serial',  'user_id' => 'int',  'body' => 'text',  'type' => 'varchar(50)',  'viewed' => 'boolean',  'date' => 'timestamp',  'Extra' => 'bytea',  'notify_online' => 'boolean',  'notify_feed' => 'boolean',  'notify_email' => 'boolean');
        $this->defaultValues = array( 'viewed' => 'false',  'notify_online' => 'true',  'notify_feed' => 'false',  'notify_email' => 'true');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\NotificationPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
