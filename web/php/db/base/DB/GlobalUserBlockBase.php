<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table global_user_block.
 */
class GlobalUserBlockBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='global_user_block';
        $this->peerName = 'DB\\GlobalUserBlockPeer';
        $this->primaryKeyName = 'block_id';
        $this->fieldNames = array( 'block_id' ,  'site_id' ,  'user_id' ,  'reason' ,  'date_blocked' );

        //$this->fieldDefaultValues=
    }






    public function getBlockId()
    {
        return $this->getFieldValue('block_id');
    }

    public function setBlockId($v1, $raw = false)
    {
        $this->setFieldValue('block_id', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getUserId()
    {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw = false)
    {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getReason()
    {
        return $this->getFieldValue('reason');
    }

    public function setReason($v1, $raw = false)
    {
        $this->setFieldValue('reason', $v1, $raw);
    }


    public function getDateBlocked()
    {
        return $this->getFieldValue('date_blocked');
    }

    public function setDateBlocked($v1, $raw = false)
    {
        $this->setFieldValue('date_blocked', $v1, $raw);
    }
}
