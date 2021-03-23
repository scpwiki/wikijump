<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table anonymous_abuse_flag.
 */
class AnonymousAbuseFlagPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='anonymous_abuse_flag';
        $this->objectName='DB\\AnonymousAbuseFlag';
        $this->primaryKeyName = 'flag_id';
        $this->fieldNames = array( 'flag_id' ,  'user_id' ,  'address' ,  'proxy' ,  'site_id' ,  'site_valid' ,  'global_valid' );
        $this->fieldTypes = array( 'flag_id' => 'serial',  'user_id' => 'int',  'address' => 'inet',  'proxy' => 'boolean',  'site_id' => 'int',  'site_valid' => 'boolean',  'global_valid' => 'boolean');
        $this->defaultValues = array( 'proxy' => 'false',  'site_valid' => 'true',  'global_valid' => 'true');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\AnonymousAbuseFlagPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
