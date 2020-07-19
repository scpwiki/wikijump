<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table forum_thread.
 */
class ForumThreadPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='forum_thread';
        $this->objectName='DB\\ForumThread';
        $this->primaryKeyName = 'thread_id';
        $this->fieldNames = array( 'thread_id' ,  'user_id' ,  'user_string' ,  'category_id' ,  'title' ,  'description' ,  'number_posts' ,  'date_started' ,  'site_id' ,  'last_post_id' ,  'page_id' ,  'sticky' ,  'blocked' );
        $this->fieldTypes = array( 'thread_id' => 'serial',  'user_id' => 'int',  'user_string' => 'varchar(80)',  'category_id' => 'int',  'title' => 'varchar(256)',  'description' => 'varchar(1000)',  'number_posts' => 'int',  'date_started' => 'timestamp',  'site_id' => 'int',  'last_post_id' => 'int',  'page_id' => 'int',  'sticky' => 'boolean',  'blocked' => 'boolean');
        $this->defaultValues = array( 'number_posts' => '1',  'sticky' => 'false',  'blocked' => 'false');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\ForumThreadPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
