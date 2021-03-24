<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table ozone_lock.
 */
class OzoneLockBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='ozone_lock';
        $this->peerName = 'Wikidot\\DB\\OzoneLockPeer';
        $this->primaryKeyName = 'key';
        $this->fieldNames = array( 'key' );

        //$this->fieldDefaultValues=
    }






    public function getKey()
    {
        return $this->getFieldValue('key');
    }

    public function setKey($v1, $raw = false)
    {
        $this->setFieldValue('key', $v1, $raw);
    }
}
