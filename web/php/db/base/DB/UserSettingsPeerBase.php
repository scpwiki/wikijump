<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table user_settings.
 */
class UserSettingsPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='user_settings';
        $this->objectName='DB\\UserSettings';
        $this->primaryKeyName = 'user_id';
        $this->fieldNames = array( 'user_id' ,  'receive_invitations' ,  'receive_pm' ,  'receive_newsletter' ,  'receive_digest' ,  'notify_online' ,  'notify_feed' ,  'notify_email' ,  'allow_site_newsletters_default' ,  'max_sites_admin' );
        $this->fieldTypes = array( 'user_id' => 'int',  'receive_invitations' => 'boolean',  'receive_pm' => 'char(5)',  'receive_newsletter' => 'boolean',  'receive_digest' => 'boolean',  'notify_online' => 'varchar(512)',  'notify_feed' => 'varchar(512)',  'notify_email' => 'varchar(512)',  'allow_site_newsletters_default' => 'boolean',  'max_sites_admin' => 'int');
        $this->defaultValues = array( 'receive_invitations' => 'true',  'receive_pm' => 'a',  'receive_newsletter' => 'true',  'receive_digest' => 'true',  'notify_online' => '*',  'notify_feed' => '*',  'allow_site_newsletters_default' => 'true',  'max_sites_admin' => '3');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\UserSettingsPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
