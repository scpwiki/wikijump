<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table comment_revision.
 */
class CommentRevisionPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='comment_revision';
        $this->objectName='DB\\CommentRevision';
        $this->primaryKeyName = 'revision_id';
        $this->fieldNames = array( 'revision_id' ,  'comment_id' ,  'user_id' ,  'user_string' ,  'text' ,  'title' ,  'date' );
        $this->fieldTypes = array( 'revision_id' => 'serial',  'comment_id' => 'int',  'user_id' => 'int',  'user_string' => 'varchar(80)',  'text' => 'text',  'title' => 'varchar(256)',  'date' => 'timestamp');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\CommentRevisionPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
