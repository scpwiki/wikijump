<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table private_user_block.
 */
class PrivateUserBlockBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='private_user_block';
        $this->peerName = 'DB\\PrivateUserBlockPeer';
        $this->primaryKeyName = 'block_id';
        $this->fieldNames = array( 'block_id' ,  'user_id' ,  'blocked_user_id' );

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


    public function getUserId()
    {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw = false)
    {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getBlockedUserId()
    {
        return $this->getFieldValue('blocked_user_id');
    }

    public function setBlockedUserId($v1, $raw = false)
    {
        $this->setFieldValue('blocked_user_id', $v1, $raw);
    }
}
