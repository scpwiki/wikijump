<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table admin_notification.
 */
class AdminNotificationPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='admin_notification';
        $this->objectName='DB\\AdminNotification';
        $this->primaryKeyName = 'notification_id';
        $this->fieldNames = array( 'notification_id' ,  'site_id' ,  'body' ,  'type' ,  'viewed' ,  'date' ,  'extra' ,  'notify_online' ,  'notify_feed' ,  'notify_email' );
        $this->fieldTypes = array( 'notification_id' => 'serial',  'site_id' => 'int',  'body' => 'text',  'type' => 'varchar(50)',  'viewed' => 'boolean',  'date' => 'timestamp',  'extra' => 'bytea',  'notify_online' => 'boolean',  'notify_feed' => 'boolean',  'notify_email' => 'boolean');
        $this->defaultValues = array( 'viewed' => 'false',  'notify_online' => 'false',  'notify_feed' => 'false',  'notify_email' => 'false');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\AdminNotificationPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
