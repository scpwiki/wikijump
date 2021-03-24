<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table user_abuse_flag.
 */
class UserAbuseFlagPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='user_abuse_flag';
        $this->objectName='Wikidot\\DB\\UserAbuseFlag';
        $this->primaryKeyName = 'flag_id';
        $this->fieldNames = array( 'flag_id' ,  'user_id' ,  'target_user_id' ,  'site_id' ,  'site_valid' ,  'global_valid' );
        $this->fieldTypes = array( 'flag_id' => 'serial',  'user_id' => 'int',  'target_user_id' => 'int',  'site_id' => 'int',  'site_valid' => 'boolean',  'global_valid' => 'boolean');
        $this->defaultValues = array( 'site_valid' => 'true',  'global_valid' => 'true');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\UserAbuseFlagPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
