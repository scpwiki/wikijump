<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table file.
 */
class FilePeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='file';
        $this->objectName='Wikidot\\DB\\File';
        $this->primaryKeyName = 'file_id';
        $this->fieldNames = array( 'file_id' ,  'page_id' ,  'site_id' ,  'filename' ,  'mimetype' ,  'description' ,  'description_short' ,  'comment' ,  'size' ,  'date_added' ,  'user_id' ,  'user_string' ,  'has_resized' );
        $this->fieldTypes = array( 'file_id' => 'serial',  'page_id' => 'int',  'site_id' => 'int',  'filename' => 'varchar(100)',  'mimetype' => 'varchar(100)',  'description' => 'varchar(200)',  'description_short' => 'varchar(200)',  'comment' => 'varchar(400)',  'size' => 'int',  'date_added' => 'timestamp',  'user_id' => 'int',  'user_string' => 'varchar(80)',  'has_resized' => 'boolean');
        $this->defaultValues = array( 'has_resized' => 'false');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\FilePeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
