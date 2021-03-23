<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table unique_string_broker.
 */
class UniqueStringBrokerBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='unique_string_broker';
        $this->peerName = 'Wikidot\\DB\\UniqueStringBrokerPeer';
        $this->primaryKeyName = '';
        $this->fieldNames = array( 'last_index' );

        //$this->fieldDefaultValues=
    }






    public function getLastIndex()
    {
        return $this->getFieldValue('last_index');
    }

    public function setLastIndex($v1, $raw = false)
    {
        $this->setFieldValue('last_index', $v1, $raw);
    }
}
