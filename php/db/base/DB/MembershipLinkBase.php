<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table membership_link.
 */
class MembershipLinkBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='membership_link';
        $this->peerName = 'DB\\MembershipLinkPeer';
        $this->primaryKeyName = 'link_id';
        $this->fieldNames = array( 'link_id' ,  'site_id' ,  'by_user_id' ,  'user_id' ,  'date' ,  'type' );

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


    public function getByUserId()
    {
        return $this->getFieldValue('by_user_id');
    }

    public function setByUserId($v1, $raw = false)
    {
        $this->setFieldValue('by_user_id', $v1, $raw);
    }


    public function getUserId()
    {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw = false)
    {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getDate()
    {
        return $this->getFieldValue('date');
    }

    public function setDate($v1, $raw = false)
    {
        $this->setFieldValue('date', $v1, $raw);
    }


    public function getType()
    {
        return $this->getFieldValue('type');
    }

    public function setType($v1, $raw = false)
    {
        $this->setFieldValue('type', $v1, $raw);
    }
}
