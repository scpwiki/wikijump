<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table log_event.
 */
class LogEventPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='log_event';
        $this->objectName='Wikidot\\DB\\LogEvent';
        $this->primaryKeyName = 'event_id';
        $this->fieldNames = array( 'event_id' ,  'date' ,  'user_id' ,  'ip' ,  'proxy' ,  'type' ,  'site_id' ,  'page_id' ,  'revision_id' ,  'thread_id' ,  'post_id' ,  'user_agent' ,  'text' );
        $this->fieldTypes = array( 'event_id' => 'bigserial',  'date' => 'timestamp',  'user_id' => 'int',  'ip' => 'inet',  'proxy' => 'inet',  'type' => 'varchar(256)',  'site_id' => 'int',  'page_id' => 'int',  'revision_id' => 'int',  'thread_id' => 'int',  'post_id' => 'int',  'user_agent' => 'varchar(512)',  'text' => 'text');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\LogEventPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
