<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table category.
 */
class CategoryPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='category';
        $this->objectName='Wikidot\\DB\\Category';
        $this->primaryKeyName = 'category_id';
        $this->fieldNames = array( 'category_id' ,  'site_id' ,  'name' ,  'theme_default' ,  'theme_id' ,  'theme_external_url' ,  'permissions_default' ,  'permissions' ,  'license_inherits' ,  'license_id' ,  'nav_default' ,  'top_bar_page_name' ,  'side_bar_page_name' ,  'template_id' ,  'per_page_discussion' ,  'per_page_discussion_default' ,  'rating' ,  'category_template_id' ,  'autonumerate' ,  'page_title_template' );
        $this->fieldTypes = array( 'category_id' => 'serial',  'site_id' => 'int',  'name' => 'varchar(80)',  'theme_default' => 'boolean',  'theme_id' => 'int',  'theme_external_url' => 'varchar(512)',  'permissions_default' => 'boolean',  'permissions' => 'varchar(200)',  'license_inherits' => 'boolean',  'license_id' => 'varchar(20)', 'nav_default' => 'boolean',  'top_bar_page_name' => 'varchar(128)',  'side_bar_page_name' => 'varchar(128)',  'template_id' => 'int',  'per_page_discussion' => 'boolean',  'per_page_discussion_default' => 'boolean',  'rating' => 'varchar(10)',  'category_template_id' => 'int',  'autonumerate' => 'boolean',  'page_title_template' => 'varchar(256)' );
        $this->defaultValues = array( 'theme_default' => 'true',  'permissions_default' => 'true',  'license_inherits' => 'true',  'nav_default' => 'true',  'per_page_discussion_default' => 'true',  'autonumerate' => 'false' );
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            self::$peerInstance = new CategoryPeer();
        }
        return self::$peerInstance;
    }
}
