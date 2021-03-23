<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table page_inclusion.
 */
class PageInclusionBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='page_inclusion';
        $this->peerName = 'DB\\PageInclusionPeer';
        $this->primaryKeyName = 'inclusion_id';
        $this->fieldNames = array( 'inclusion_id' ,  'site_id' ,  'including_page_id' ,  'included_page_id' ,  'included_page_name' );

        //$this->fieldDefaultValues=
    }






    public function getInclusionId()
    {
        return $this->getFieldValue('inclusion_id');
    }

    public function setInclusionId($v1, $raw = false)
    {
        $this->setFieldValue('inclusion_id', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getIncludingPageId()
    {
        return $this->getFieldValue('including_page_id');
    }

    public function setIncludingPageId($v1, $raw = false)
    {
        $this->setFieldValue('including_page_id', $v1, $raw);
    }


    public function getIncludedPageId()
    {
        return $this->getFieldValue('included_page_id');
    }

    public function setIncludedPageId($v1, $raw = false)
    {
        $this->setFieldValue('included_page_id', $v1, $raw);
    }


    public function getIncludedPageName()
    {
        return $this->getFieldValue('included_page_name');
    }

    public function setIncludedPageName($v1, $raw = false)
    {
        $this->setFieldValue('included_page_name', $v1, $raw);
    }
}
