<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;
use Ozone\Framework\Database\Criteria;

/**
 * Base Class mapped to the database table page.
 */
class PageBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='page';
        $this->peerName = 'Wikidot\\DB\\PagePeer';
        $this->primaryKeyName = 'page_id';
        $this->fieldNames = array( 'page_id' ,  'site_id' ,  'category_id' ,  'parent_page_id' ,  'revision_id' ,  'source_id' ,  'metadata_id' ,  'revision_number' ,  'title' ,  'unix_name' ,  'date_created' ,  'date_last_edited' ,  'last_edit_user_id' ,  'last_edit_user_string' ,  'thread_id' ,  'owner_user_id' ,  'blocked' ,  'rate' , 'tags' );

        //$this->fieldDefaultValues=
    }



    public function getSite()
    {
        if (is_array($this->prefetched)) {
            if (in_array('site', $this->prefetched)) {
                if (in_array('site', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['site'];
                } else {
                    $obj = new Site($this->sourceRow);
                    $obj->setNew(false);
                    //$obj->prefetched = $this->prefetched;
                    //$obj->sourceRow = $this->sourceRow;
                    $this->prefetchedObjects['site'] = $obj;
                    return $obj;
                }
            }
        }
                $foreignPeerClassName = 'Wikidot\\DB\\SitePeer';
                $fpeer = new $foreignPeerClassName();

                $criteria = new Criteria();

                $criteria->add("site_id", $this->fieldValues['site_id']);

                $result = $fpeer->selectOneByCriteria($criteria);
                return $result;
    }

    public function setSite($primaryObject)
    {
        $this->fieldValues['site_id'] = $primaryObject->getFieldValue('site_id');
    }



    public function getPageId()
    {
        return $this->getFieldValue('page_id');
    }

    public function setPageId($v1, $raw = false)
    {
        $this->setFieldValue('page_id', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getCategoryId()
    {
        return $this->getFieldValue('category_id');
    }

    public function setCategoryId($v1, $raw = false)
    {
        $this->setFieldValue('category_id', $v1, $raw);
    }


    public function getParentPageId()
    {
        return $this->getFieldValue('parent_page_id');
    }

    public function setParentPageId($v1, $raw = false)
    {
        $this->setFieldValue('parent_page_id', $v1, $raw);
    }


    public function getRevisionId()
    {
        return $this->getFieldValue('revision_id');
    }

    public function setRevisionId($v1, $raw = false)
    {
        $this->setFieldValue('revision_id', $v1, $raw);
    }


    public function getSourceId()
    {
        return $this->getFieldValue('source_id');
    }

    public function setSourceId($v1, $raw = false)
    {
        $this->setFieldValue('source_id', $v1, $raw);
    }


    public function getMetadataId()
    {
        return $this->getFieldValue('metadata_id');
    }

    public function setMetadataId($v1, $raw = false)
    {
        $this->setFieldValue('metadata_id', $v1, $raw);
    }


    public function getRevisionNumber()
    {
        return $this->getFieldValue('revision_number');
    }

    public function setRevisionNumber($v1, $raw = false)
    {
        $this->setFieldValue('revision_number', $v1, $raw);
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


    public function getDateCreated()
    {
        return $this->getFieldValue('date_created');
    }

    public function setDateCreated($v1, $raw = false)
    {
        $this->setFieldValue('date_created', $v1, $raw);
    }


    public function getDateLastEdited()
    {
        return $this->getFieldValue('date_last_edited');
    }

    public function setDateLastEdited($v1, $raw = false)
    {
        $this->setFieldValue('date_last_edited', $v1, $raw);
    }


    public function getLastEditUserId()
    {
        return $this->getFieldValue('last_edit_user_id');
    }

    public function setLastEditUserId($v1, $raw = false)
    {
        $this->setFieldValue('last_edit_user_id', $v1, $raw);
    }


    public function getLastEditUserString()
    {
        return $this->getFieldValue('last_edit_user_string');
    }

    public function setLastEditUserString($v1, $raw = false)
    {
        $this->setFieldValue('last_edit_user_string', $v1, $raw);
    }


    public function getThreadId()
    {
        return $this->getFieldValue('thread_id');
    }

    public function setThreadId($v1, $raw = false)
    {
        $this->setFieldValue('thread_id', $v1, $raw);
    }


    public function getOwnerUserId()
    {
        return $this->getFieldValue('owner_user_id');
    }

    public function setOwnerUserId($v1, $raw = false)
    {
        $this->setFieldValue('owner_user_id', $v1, $raw);
    }


    public function getBlocked()
    {
        return $this->getFieldValue('blocked');
    }

    public function setBlocked($v1, $raw = false)
    {
        $this->setFieldValue('blocked', $v1, $raw);
    }


    public function getRate()
    {
        return $this->getFieldValue('rate');
    }

    public function setRate($v1, $raw = false)
    {
        $this->setFieldValue('rate', $v1, $raw);
    }

    public function setTags($v1, $raw = false)
    {
        $this->setFieldValue('tags', $v1, $raw);
    }
}
