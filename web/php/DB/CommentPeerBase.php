<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table comment.
 */
class CommentPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='comment';
        $this->objectName='DB\\Comment';
        $this->primaryKeyName = 'comment_id';
        $this->fieldNames = array( 'comment_id' ,  'page_id' ,  'parent_id' ,  'user_id' ,  'user_string' ,  'title' ,  'text' ,  'date_posted' ,  'site_id' ,  'revision_number' ,  'revision_id' ,  'date_last_edited' ,  'edited_user_id' ,  'edited_user_string' );
        $this->fieldTypes = array( 'comment_id' => 'serial',  'page_id' => 'int',  'parent_id' => 'int',  'user_id' => 'int',  'user_string' => 'varchar(80)',  'title' => 'varchar(256)',  'text' => 'text',  'date_posted' => 'timestamp',  'site_id' => 'int',  'revision_number' => 'int',  'revision_id' => 'int',  'date_last_edited' => 'timestamp',  'edited_user_id' => 'int',  'edited_user_string' => 'varchar(80)');
        $this->defaultValues = array( 'revision_number' => '0');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\CommentPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
