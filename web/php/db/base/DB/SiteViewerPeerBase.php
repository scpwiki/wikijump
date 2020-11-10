<?php


namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table site_viewer.
 */
class SiteViewerPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='site_viewer';
        $this->objectName='DB\\SiteViewer';
        $this->primaryKeyName = 'viewer_id';
        $this->fieldNames = array( 'viewer_id' ,  'site_id' ,  'user_id' );
        $this->fieldTypes = array( 'viewer_id' => 'serial',  'site_id' => 'int',  'user_id' => 'int');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\SiteViewerPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
