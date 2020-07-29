<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table forum_post.
 */
class ForumPostPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='forum_post';
        $this->objectName='DB\\ForumPost';
        $this->primaryKeyName = 'post_id';
        $this->fieldNames = array( 'post_id' ,  'thread_id' ,  'parent_id' ,  'user_id' ,  'user_string' ,  'title' ,  'text' ,  'date_posted' ,  'site_id' ,  'revision_number' ,  'revision_id' ,  'date_last_edited' ,  'edited_user_id' ,  'edited_user_string' );
        $this->fieldTypes = array( 'post_id' => 'serial',  'thread_id' => 'int',  'parent_id' => 'int',  'user_id' => 'int',  'user_string' => 'varchar(80)',  'title' => 'varchar(256)',  'text' => 'text',  'date_posted' => 'timestamp',  'site_id' => 'int',  'revision_number' => 'int',  'revision_id' => 'int',  'date_last_edited' => 'timestamp',  'edited_user_id' => 'int',  'edited_user_string' => 'varchar(80)');
        $this->defaultValues = array( 'revision_number' => '0');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\ForumPostPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
