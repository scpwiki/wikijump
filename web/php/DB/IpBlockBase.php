<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table ip_block.
 */
class IpBlockBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='ip_block';
        $this->peerName = 'Wikidot\\DB\\IpBlockPeer';
        $this->primaryKeyName = 'block_id';
        $this->fieldNames = array( 'block_id' ,  'site_id' ,  'ip' ,  'flag_proxy' ,  'reason' ,  'date_blocked' );

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


    public function getIp()
    {
        return $this->getFieldValue('ip');
    }

    public function setIp($v1, $raw = false)
    {
        $this->setFieldValue('ip', $v1, $raw);
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


    public function getDateBlocked()
    {
        return $this->getFieldValue('date_blocked');
    }

    public function setDateBlocked($v1, $raw = false)
    {
        $this->setFieldValue('date_blocked', $v1, $raw);
    }
}
