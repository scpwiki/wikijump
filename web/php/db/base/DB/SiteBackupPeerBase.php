<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table site_backup.
 */
class SiteBackupPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='site_backup';
        $this->objectName='DB\\SiteBackup';
        $this->primaryKeyName = 'backup_id';
        $this->fieldNames = array( 'backup_id' ,  'site_id' ,  'status' ,  'backup_source' ,  'backup_files' ,  'date' ,  'rand' );
        $this->fieldTypes = array( 'backup_id' => 'serial',  'site_id' => 'int',  'status' => 'varchar(50)',  'backup_source' => 'boolean',  'backup_files' => 'boolean',  'date' => 'timestamp',  'rand' => 'varchar(100)');
        $this->defaultValues = array( 'backup_source' => 'true',  'backup_files' => 'true');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\SiteBackupPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
