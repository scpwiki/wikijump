<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table unique_string_broker.
 */
class UniqueStringBrokerPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='unique_string_broker';
        $this->objectName='Wikidot\\DB\\UniqueStringBroker';
        $this->primaryKeyName = '';
        $this->fieldNames = array( 'last_index' );
        $this->fieldTypes = array( 'last_index' => 'int');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\UniqueStringBrokerPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
