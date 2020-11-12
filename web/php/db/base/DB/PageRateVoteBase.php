<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table page_rate_vote.
 */
class PageRateVoteBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='page_rate_vote';
        $this->peerName = 'DB\\PageRateVotePeer';
        $this->primaryKeyName = 'rate_id';
        $this->fieldNames = array( 'rate_id' ,  'user_id' ,  'page_id' ,  'rate' ,  'date' );

        //$this->fieldDefaultValues=
    }






    public function getRateId()
    {
        return $this->getFieldValue('rate_id');
    }

    public function setRateId($v1, $raw = false)
    {
        $this->setFieldValue('rate_id', $v1, $raw);
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


    public function getRate()
    {
        return $this->getFieldValue('rate');
    }

    public function setRate($v1, $raw = false)
    {
        $this->setFieldValue('rate', $v1, $raw);
    }


    public function getDate()
    {
        return $this->getFieldValue('date');
    }

    public function setDate($v1, $raw = false)
    {
        $this->setFieldValue('date', $v1, $raw);
    }
}
