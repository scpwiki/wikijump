<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table page_compiled.
 */
class PageCompiledBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='page_compiled';
        $this->peerName = 'DB\\PageCompiledPeer';
        $this->primaryKeyName = 'page_id';
        $this->fieldNames = array( 'page_id' ,  'text' ,  'date_compiled' );

        //$this->fieldDefaultValues=
    }






    public function getPageId()
    {
        return $this->getFieldValue('page_id');
    }

    public function setPageId($v1, $raw = false)
    {
        $this->setFieldValue('page_id', $v1, $raw);
    }


    public function getText()
    {
        return $this->getFieldValue('text');
    }

    public function setText($v1, $raw = false)
    {
        $this->setFieldValue('text', $v1, $raw);
    }


    public function getDateCompiled()
    {
        return $this->getFieldValue('date_compiled');
    }

    public function setDateCompiled($v1, $raw = false)
    {
        $this->setFieldValue('date_compiled', $v1, $raw);
    }
}
