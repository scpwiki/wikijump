<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table profile.
 */
class ProfilePeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='profile';
        $this->objectName='Wikidot\\DB\\Profile';
        $this->primaryKeyName = 'user_id';
        $this->fieldNames = array( 'user_id' ,  'real_name' ,  'pronouns' ,  'birthday_day' ,  'birthday_month' ,  'birthday_year' ,  'about' ,  'location' ,  'website' ,  'im_icq' ,  'im_jabber' ,  'change_screen_name_count' );
        $this->fieldTypes = array( 'user_id' => 'int',  'real_name' => 'varchar(70)',  'pronouns' => 'varchar(30)',  'birthday_day' => 'int',  'birthday_month' => 'int',  'birthday_year' => 'int',  'about' => 'text',  'location' => 'varchar(70)',  'website' => 'varchar(100)',  'im_icq' => 'varchar(100)',  'im_jabber' => 'varchar(100)',  'change_screen_name_count' => 'int');
        $this->defaultValues = array( 'change_screen_name_count' => '0');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\ProfilePeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
