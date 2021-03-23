<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table domain_redirect.
 */
class DomainRedirectBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='domain_redirect';
        $this->peerName = 'DB\\DomainRedirectPeer';
        $this->primaryKeyName = 'redirect_id';
        $this->fieldNames = array( 'redirect_id' ,  'site_id' ,  'url' );

        //$this->fieldDefaultValues=
    }






    public function getRedirectId()
    {
        return $this->getFieldValue('redirect_id');
    }

    public function setRedirectId($v1, $raw = false)
    {
        $this->setFieldValue('redirect_id', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getUrl()
    {
        return $this->getFieldValue('url');
    }

    public function setUrl($v1, $raw = false)
    {
        $this->setFieldValue('url', $v1, $raw);
    }
}
