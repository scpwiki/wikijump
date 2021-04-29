<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table ozone_user.
 */
class OzoneUserPeerBase extends BaseDBPeer
{
    public static OzoneUserPeer $peerInstance;

    protected function internalInit()
    {
        $this->tableName='users';
        $this->objectName='Wikidot\\DB\\OzoneUser';
        $this->primaryKeyName = 'id';
        $this->fieldNames = [
            'id',
            'username',
            'unix_name',
            'username_changes',
            'email',
            'email_verified_at',
            'password',
            'two_factor_secret',
            'two_factor_recovery_codes',
            'remember_token',
            'language',
            'karma_points',
            'karma_level',
            'real_name',
            'pronouns',
            'dob',
            'bio',
            'about_page',
            'created_at',
            'updated_at',
            'deleted_at'
        ];
        $this->fieldTypes = [
            'id' => 'serial',
            'username' => '',
            'unix_name' => '',
            'username_changes' => '',
            'email' => '',
            'email_verified_at' => '',
            'password' => '',
            'two_factor_secret' => '',
            'two_factor_recovery_codes' => '',
            'remember_token' => '',
            'language' => '',
            'karma_points' => '',
            'karma_level' => '',
            'real_name' => '',
            'pronouns' => '',
            'dob' => '',
            'bio' => '',
            'about_page' => '',
            'created_at' => '',
            'updated_at' => '',
            'deleted_at' => ''
        ];
        $this->defaultValues = [
            'language' => env('DEFAULT_LANGUAGE', 'en')
        ];
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\OzoneUserPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
