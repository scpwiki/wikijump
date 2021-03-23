<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table files_event.
 */
class FilesEventPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='files_event';
        $this->objectName='Wikidot\\DB\\FilesEvent';
        $this->primaryKeyName = 'file_event_id';
        $this->fieldNames = array( 'file_event_id' ,  'filename' ,  'date' ,  'user_id' ,  'user_string' ,  'action' ,  'action_extra' );
        $this->fieldTypes = array( 'file_event_id' => 'serial',  'filename' => 'varchar(100)',  'date' => 'timestamp',  'user_id' => 'int',  'user_string' => 'varchar(80)',  'action' => 'varchar(80)',  'action_extra' => 'varchar(80)');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\FilesEventPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
