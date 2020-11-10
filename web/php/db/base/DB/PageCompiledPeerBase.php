<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table page_compiled.
 */
class PageCompiledPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='page_compiled';
        $this->objectName='DB\\PageCompiled';
        $this->primaryKeyName = 'page_id';
        $this->fieldNames = array( 'page_id' ,  'text' ,  'date_compiled' );
        $this->fieldTypes = array( 'page_id' => 'int',  'text' => 'text',  'date_compiled' => 'timestamp');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            self::$peerInstance = new PageCompiledPeer();
        }
        return self::$peerInstance;
    }
}
