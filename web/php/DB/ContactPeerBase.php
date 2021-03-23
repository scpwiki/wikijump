<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table contact.
 */
class ContactPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='contact';
        $this->objectName='Wikidot\\DB\\Contact';
        $this->primaryKeyName = 'contact_id';
        $this->fieldNames = array( 'contact_id' ,  'user_id' ,  'target_user_id' );
        $this->fieldTypes = array( 'contact_id' => 'serial',  'user_id' => 'int',  'target_user_id' => 'int');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\ContactPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
