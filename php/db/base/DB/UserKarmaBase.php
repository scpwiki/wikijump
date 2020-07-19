<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table user_karma.
 */
class UserKarmaBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='user_karma';
        $this->peerName = 'DB\\UserKarmaPeer';
        $this->primaryKeyName = 'user_id';
        $this->fieldNames = array( 'user_id' ,  'points' ,  'level' );

        //$this->fieldDefaultValues=
    }






    public function getUserId()
    {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw = false)
    {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getPoints()
    {
        return $this->getFieldValue('points');
    }

    public function setPoints($v1, $raw = false)
    {
        $this->setFieldValue('points', $v1, $raw);
    }


    public function getLevel()
    {
        return $this->getFieldValue('level');
    }

    public function setLevel($v1, $raw = false)
    {
        $this->setFieldValue('level', $v1, $raw);
    }
}
