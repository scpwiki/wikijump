<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table watched_page.
 */
class WatchedPageBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='watched_page';
        $this->peerName = 'DB\\WatchedPagePeer';
        $this->primaryKeyName = 'watched_id';
        $this->fieldNames = array( 'watched_id' ,  'user_id' ,  'page_id' );

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


    public function getPageId()
    {
        return $this->getFieldValue('page_id');
    }

    public function setPageId($v1, $raw = false)
    {
        $this->setFieldValue('page_id', $v1, $raw);
    }
}
