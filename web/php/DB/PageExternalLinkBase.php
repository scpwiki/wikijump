<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table page_external_link.
 */
class PageExternalLinkBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='page_external_link';
        $this->peerName = 'Wikidot\\DB\\PageExternalLinkPeer';
        $this->primaryKeyName = 'link_id';
        $this->fieldNames = array( 'link_id' ,  'site_id' ,  'page_id' ,  'to_url' ,  'pinged' ,  'ping_status' ,  'date' );

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


    public function getPageId()
    {
        return $this->getFieldValue('page_id');
    }

    public function setPageId($v1, $raw = false)
    {
        $this->setFieldValue('page_id', $v1, $raw);
    }


    public function getToUrl()
    {
        return $this->getFieldValue('to_url');
    }

    public function setToUrl($v1, $raw = false)
    {
        $this->setFieldValue('to_url', $v1, $raw);
    }


    public function getPinged()
    {
        return $this->getFieldValue('pinged');
    }

    public function setPinged($v1, $raw = false)
    {
        $this->setFieldValue('pinged', $v1, $raw);
    }


    public function getPingStatus()
    {
        return $this->getFieldValue('ping_status');
    }

    public function setPingStatus($v1, $raw = false)
    {
        $this->setFieldValue('ping_status', $v1, $raw);
    }


    public function getDate()
    {
        return $this->getFieldValue('date');
    }

    public function setDate($v1, $raw = false)
    {
        $this->setFieldValue('date', $v1, $raw);
    }
}
