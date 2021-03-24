<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table global_ip_block.
 */
class GlobalIpBlockBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='global_ip_block';
        $this->peerName = 'Wikidot\\DB\\GlobalIpBlockPeer';
        $this->primaryKeyName = 'block_id';
        $this->fieldNames = array( 'block_id' ,  'address' ,  'flag_proxy' ,  'reason' ,  'flag_total' ,  'date_blocked' );

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


    public function getAddress()
    {
        return $this->getFieldValue('address');
    }

    public function setAddress($v1, $raw = false)
    {
        $this->setFieldValue('address', $v1, $raw);
    }


    public function getFlagProxy()
    {
        return $this->getFieldValue('flag_proxy');
    }

    public function setFlagProxy($v1, $raw = false)
    {
        $this->setFieldValue('flag_proxy', $v1, $raw);
    }


    public function getReason()
    {
        return $this->getFieldValue('reason');
    }

    public function setReason($v1, $raw = false)
    {
        $this->setFieldValue('reason', $v1, $raw);
    }


    public function getFlagTotal()
    {
        return $this->getFieldValue('flag_total');
    }

    public function setFlagTotal($v1, $raw = false)
    {
        $this->setFieldValue('flag_total', $v1, $raw);
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
