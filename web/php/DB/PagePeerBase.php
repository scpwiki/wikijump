<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table page.
 */
class PagePeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='page';
        $this->objectName='Wikidot\\DB\\Page';
        $this->primaryKeyName = 'page_id';
        $this->fieldNames = array( 'page_id' ,  'site_id' ,  'category_id' ,  'parent_page_id' ,  'revision_id' ,   'metadata_id' ,  'revision_number' ,  'title' ,  'unix_name' ,  'date_created' ,  'date_last_edited' ,  'last_edit_user_id' ,  'last_edit_user_string' ,  'thread_id' ,  'owner_user_id' ,  'blocked' ,  'rate' , 'tags' );
        $this->fieldTypes = array( 'page_id' => 'serial',  'site_id' => 'int',  'category_id' => 'int',  'parent_page_id' => 'int',  'revision_id' => 'int',  'metadata_id' => 'int',  'revision_number' => 'int',  'title' => 'varchar(256)',  'unix_name' => 'varchar(256)',  'date_created' => 'timestamp',  'date_last_edited' => 'timestamp',  'last_edit_user_id' => 'int',  'last_edit_user_string' => 'varchar(80)',  'thread_id' => 'int',  'owner_user_id' => 'int',  'blocked' => 'boolean',  'rate' => 'int', 'tags' => 'jsonb');
        $this->defaultValues = array( 'revision_number' => '0',  'blocked' => 'false',  'rate' => '0',  'tags' => '[]');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            self::$peerInstance = new PagePeer();
        }
        return self::$peerInstance;
    }
}
