<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table front_forum_feed.
 */
class FrontForumFeedPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='front_forum_feed';
        $this->objectName='Wikidot\\DB\\FrontForumFeed';
        $this->primaryKeyName = 'feed_id';
        $this->fieldNames = array( 'feed_id' ,  'page_id' ,  'title' ,  'label' ,  'description' ,  'categories' ,  'parmhash' ,  'site_id' );
        $this->fieldTypes = array( 'feed_id' => 'serial',  'page_id' => 'int',  'title' => 'varchar(256)',  'label' => 'varchar(90)',  'description' => 'varchar(256)',  'categories' => 'varchar(100)',  'parmhash' => 'varchar(100)',  'site_id' => 'int');
        $this->defaultValues = array();
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = 'Wikidot\\DB\\FrontForumFeedPeer';
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
