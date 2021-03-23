<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table page_link.
 */
class PageLinkBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='page_link';
        $this->peerName = 'Wikidot\\DB\\PageLinkPeer';
        $this->primaryKeyName = 'link_id';
        $this->fieldNames = array( 'link_id' ,  'site_id' ,  'from_page_id' ,  'to_page_id' ,  'to_page_name' );

        //$this->fieldDefaultValues=
    }






    public function getLinkId()
    {
        return $this->getFieldValue('link_id');
    }

    public function setLinkId($v1, $raw = false)
    {
        $this->setFieldValue('link_id', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getFromPageId()
    {
        return $this->getFieldValue('from_page_id');
    }

    public function setFromPageId($v1, $raw = false)
    {
        $this->setFieldValue('from_page_id', $v1, $raw);
    }


    public function getToPageId()
    {
        return $this->getFieldValue('to_page_id');
    }

    public function setToPageId($v1, $raw = false)
    {
        $this->setFieldValue('to_page_id', $v1, $raw);
    }


    public function getToPageName()
    {
        return $this->getFieldValue('to_page_name');
    }

    public function setToPageName($v1, $raw = false)
    {
        $this->setFieldValue('to_page_name', $v1, $raw);
    }
}
