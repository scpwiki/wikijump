<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table profile.
 */
class ProfilePeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='profile';
        $this->objectName='DB\\Profile';
        $this->primaryKeyName = 'user_id';
        $this->fieldNames = array( 'user_id' ,  'real_name' ,  'gender' ,  'birthday_day' ,  'birthday_month' ,  'birthday_year' ,  'about' ,  'location' ,  'website' ,  'im_aim' ,  'im_gadu_gadu' ,  'im_google_talk' ,  'im_icq' ,  'im_jabber' ,  'im_msn' ,  'im_yahoo' ,  'change_screen_name_count' );
        $this->fieldTypes = array( 'user_id' => 'int',  'real_name' => 'varchar(70)',  'gender' => 'char(1)',  'birthday_day' => 'int',  'birthday_month' => 'int',  'birthday_year' => 'int',  'about' => 'text',  'location' => 'varchar(70)',  'website' => 'varchar(100)',  'im_aim' => 'varchar(100)',  'im_gadu_gadu' => 'varchar(100)',  'im_google_talk' => 'varchar(100)',  'im_icq' => 'varchar(100)',  'im_jabber' => 'varchar(100)',  'im_msn' => 'varchar(100)',  'im_yahoo' => 'varchar(100)',  'change_screen_name_count' => 'int');
        $this->defaultValues = array( 'change_screen_name_count' => '0');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\ProfilePeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
