<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table watched_page.
 */
class WatchedPagePeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='watched_page';
        $this->objectName='Wikidot\\DB\\WatchedPage';
        $this->primaryKeyName = 'watched_id';
        $this->fieldNames = array( 'watched_id' ,  'user_id' ,  'page_id' );
        $this->fieldTypes = array( 'watched_id' => 'serial',  'user_id' => 'int',  'page_id' => 'int');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\WatchedPagePeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
