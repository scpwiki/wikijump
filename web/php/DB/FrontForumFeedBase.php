<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table front_forum_feed.
 */
class FrontForumFeedBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='front_forum_feed';
        $this->peerName = 'DB\\FrontForumFeedPeer';
        $this->primaryKeyName = 'feed_id';
        $this->fieldNames = array( 'feed_id' ,  'page_id' ,  'title' ,  'label' ,  'description' ,  'categories' ,  'parmhash' ,  'site_id' );

        //$this->fieldDefaultValues=
    }






    public function getFeedId()
    {
        return $this->getFieldValue('feed_id');
    }

    public function setFeedId($v1, $raw = false)
    {
        $this->setFieldValue('feed_id', $v1, $raw);
    }


    public function getPageId()
    {
        return $this->getFieldValue('page_id');
    }

    public function setPageId($v1, $raw = false)
    {
        $this->setFieldValue('page_id', $v1, $raw);
    }


    public function getTitle()
    {
        return $this->getFieldValue('title');
    }

    public function setTitle($v1, $raw = false)
    {
        $this->setFieldValue('title', $v1, $raw);
    }


    public function getLabel()
    {
        return $this->getFieldValue('label');
    }

    public function setLabel($v1, $raw = false)
    {
        $this->setFieldValue('label', $v1, $raw);
    }


    public function getDescription()
    {
        return $this->getFieldValue('description');
    }

    public function setDescription($v1, $raw = false)
    {
        $this->setFieldValue('description', $v1, $raw);
    }


    public function getCategories()
    {
        return $this->getFieldValue('categories');
    }

    public function setCategories($v1, $raw = false)
    {
        $this->setFieldValue('categories', $v1, $raw);
    }


    public function getParmhash()
    {
        return $this->getFieldValue('parmhash');
    }

    public function setParmhash($v1, $raw = false)
    {
        $this->setFieldValue('parmhash', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }
}
