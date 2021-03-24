<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table theme_preview.
 */
class ThemePreviewPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='theme_preview';
        $this->objectName='Wikidot\\DB\\ThemePreview';
        $this->primaryKeyName = 'theme_id';
        $this->fieldNames = array( 'theme_id' ,  'body' );
        $this->fieldTypes = array( 'theme_id' => 'int',  'body' => 'text');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\ThemePreviewPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
