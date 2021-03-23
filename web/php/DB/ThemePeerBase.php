<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table theme.
 */
class ThemePeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='theme';
        $this->objectName='Wikidot\\DB\\Theme';
        $this->primaryKeyName = 'theme_id';
        $this->fieldNames = array( 'theme_id' ,  'name' ,  'unix_name' ,  'abstract' ,  'extends_theme_id' ,  'variant_of_theme_id' ,  'custom' ,  'site_id' ,  'use_side_bar' ,  'use_top_bar' ,  'sort_index' ,  'sync_page_name' ,  'revision_number' );
        $this->fieldTypes = array( 'theme_id' => 'serial',  'name' => 'varchar(100)',  'unix_name' => 'varchar(100)',  'abstract' => 'boolean',  'extends_theme_id' => 'int',  'variant_of_theme_id' => 'int',  'custom' => 'boolean',  'site_id' => 'int',  'use_side_bar' => 'boolean',  'use_top_bar' => 'boolean',  'sort_index' => 'int',  'sync_page_name' => 'varchar(100)',  'revision_number' => 'int');
        $this->defaultValues = array( 'abstract' => 'false',  'custom' => 'false',  'use_side_bar' => 'true',  'use_top_bar' => 'true',  'sort_index' => '0',  'revision_number' => '0');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            self::$peerInstance = new ThemePeer();
        }
        return self::$peerInstance;
    }
}
