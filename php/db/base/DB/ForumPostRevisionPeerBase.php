<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table forum_post_revision.
 */
class ForumPostRevisionPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='forum_post_revision';
        $this->objectName='DB\\ForumPostRevision';
        $this->primaryKeyName = 'revision_id';
        $this->fieldNames = array( 'revision_id' ,  'post_id' ,  'user_id' ,  'user_string' ,  'text' ,  'title' ,  'date' );
        $this->fieldTypes = array( 'revision_id' => 'serial',  'post_id' => 'int',  'user_id' => 'int',  'user_string' => 'varchar(80)',  'text' => 'text',  'title' => 'varchar(256)',  'date' => 'timestamp');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\ForumPostRevisionPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
