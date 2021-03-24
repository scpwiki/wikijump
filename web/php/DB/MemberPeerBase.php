<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table member.
 */
class MemberPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='member';
        $this->objectName='Wikidot\\DB\\Member';
        $this->primaryKeyName = 'member_id';
        $this->fieldNames = array( 'member_id' ,  'site_id' ,  'user_id' ,  'date_joined' ,  'allow_newsletter' );
        $this->fieldTypes = array( 'member_id' => 'serial',  'site_id' => 'int',  'user_id' => 'int',  'date_joined' => 'timestamp',  'allow_newsletter' => 'boolean');
        $this->defaultValues = array( 'allow_newsletter' => 'true');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\MemberPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
