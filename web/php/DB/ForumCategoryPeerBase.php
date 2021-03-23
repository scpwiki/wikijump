<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table forum_category.
 */
class ForumCategoryPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='forum_category';
        $this->objectName='DB\\ForumCategory';
        $this->primaryKeyName = 'category_id';
        $this->fieldNames = array( 'category_id' ,  'group_id' ,  'name' ,  'description' ,  'number_posts' ,  'number_threads' ,  'last_post_id' ,  'permissions_default' ,  'permissions' ,  'max_nest_level' ,  'sort_index' ,  'site_id' ,  'per_page_discussion' );
        $this->fieldTypes = array( 'category_id' => 'serial',  'group_id' => 'int',  'name' => 'varchar(80)',  'description' => 'text',  'number_posts' => 'int',  'number_threads' => 'int',  'last_post_id' => 'int',  'permissions_default' => 'boolean',  'permissions' => 'varchar(200)',  'max_nest_level' => 'int',  'sort_index' => 'int',  'site_id' => 'int',  'per_page_discussion' => 'boolean');
        $this->defaultValues = array( 'number_posts' => '0',  'number_threads' => '0',  'permissions_default' => 'true',  'sort_index' => '0',  'per_page_discussion' => 'false');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\ForumCategoryPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
