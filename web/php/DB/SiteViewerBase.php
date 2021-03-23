<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table site_viewer.
 */
class SiteViewerBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='site_viewer';
        $this->peerName = 'DB\\SiteViewerPeer';
        $this->primaryKeyName = 'viewer_id';
        $this->fieldNames = array( 'viewer_id' ,  'site_id' ,  'user_id' );

        //$this->fieldDefaultValues=
    }






    public function getViewerId()
    {
        return $this->getFieldValue('viewer_id');
    }

    public function setViewerId($v1, $raw = false)
    {
        $this->setFieldValue('viewer_id', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getUserId()
    {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw = false)
    {
        $this->setFieldValue('user_id', $v1, $raw);
    }
}
