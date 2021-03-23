<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table page_metadata.
 */
class PageMetadataBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='page_metadata';
        $this->peerName = 'DB\\PageMetadataPeer';
        $this->primaryKeyName = 'metadata_id';
        $this->fieldNames = array( 'metadata_id' ,  'parent_page_id' ,  'title' ,  'unix_name' ,  'owner_user_id' );

        //$this->fieldDefaultValues=
    }






    public function getMetadataId()
    {
        return $this->getFieldValue('metadata_id');
    }

    public function setMetadataId($v1, $raw = false)
    {
        $this->setFieldValue('metadata_id', $v1, $raw);
    }


    public function getParentPageId()
    {
        return $this->getFieldValue('parent_page_id');
    }

    public function setParentPageId($v1, $raw = false)
    {
        $this->setFieldValue('parent_page_id', $v1, $raw);
    }


    public function getTitle()
    {
        return $this->getFieldValue('title');
    }

    public function setTitle($v1, $raw = false)
    {
        $this->setFieldValue('title', $v1, $raw);
    }


    public function getUnixName()
    {
        return $this->getFieldValue('unix_name');
    }

    public function setUnixName($v1, $raw = false)
    {
        $this->setFieldValue('unix_name', $v1, $raw);
    }


    public function getOwnerUserId()
    {
        return $this->getFieldValue('owner_user_id');
    }

    public function setOwnerUserId($v1, $raw = false)
    {
        $this->setFieldValue('owner_user_id', $v1, $raw);
    }
}
