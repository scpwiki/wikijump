<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table watched_forum_thread.
 */
class WatchedForumThreadPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='watched_forum_thread';
        $this->objectName='Wikidot\\DB\\WatchedForumThread';
        $this->primaryKeyName = 'watched_id';
        $this->fieldNames = array( 'watched_id' ,  'user_id' ,  'thread_id' );
        $this->fieldTypes = array( 'watched_id' => 'serial',  'user_id' => 'int',  'thread_id' => 'int');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\WatchedForumThreadPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
