<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table watched_forum_thread.
 */
class WatchedForumThreadBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='watched_forum_thread';
        $this->peerName = 'Wikidot\\DB\\WatchedForumThreadPeer';
        $this->primaryKeyName = 'watched_id';
        $this->fieldNames = array( 'watched_id' ,  'user_id' ,  'thread_id' );

        //$this->fieldDefaultValues=
    }






    public function getWatchedId()
    {
        return $this->getFieldValue('watched_id');
    }

    public function setWatchedId($v1, $raw = false)
    {
        $this->setFieldValue('watched_id', $v1, $raw);
    }


    public function getUserId()
    {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw = false)
    {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getThreadId()
    {
        return $this->getFieldValue('thread_id');
    }

    public function setThreadId($v1, $raw = false)
    {
        $this->setFieldValue('thread_id', $v1, $raw);
    }
}
