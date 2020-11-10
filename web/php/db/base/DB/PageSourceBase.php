<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table page_source.
 */
class PageSourceBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='page_source';
        $this->peerName = 'DB\\PageSourcePeer';
        $this->primaryKeyName = 'source_id';
        $this->fieldNames = array( 'source_id' ,  'text' );

        //$this->fieldDefaultValues=
    }






    public function getSourceId()
    {
        return $this->getFieldValue('source_id');
    }

    public function setSourceId($v1, $raw = false)
    {
        $this->setFieldValue('source_id', $v1, $raw);
    }


    public function getText()
    {
        return $this->getFieldValue('text');
    }

    public function setText($v1, $raw = false)
    {
        $this->setFieldValue('text', $v1, $raw);
    }
}
