<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table forum_post.
 */
class ForumPostBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='forum_post';
        $this->peerName = 'Wikidot\\DB\\ForumPostPeer';
        $this->primaryKeyName = 'post_id';
        $this->fieldNames = array( 'post_id' ,  'thread_id' ,  'parent_id' ,  'user_id' ,  'user_string' ,  'title' ,  'text' ,  'date_posted' ,  'site_id' ,  'revision_number' ,  'revision_id' ,  'date_last_edited' ,  'edited_user_id' ,  'edited_user_string' );

        //$this->fieldDefaultValues=
    }






    public function getPostId()
    {
        return $this->getFieldValue('post_id');
    }

    public function setPostId($v1, $raw = false)
    {
        $this->setFieldValue('post_id', $v1, $raw);
    }


    public function getThreadId()
    {
        return $this->getFieldValue('thread_id');
    }

    public function setThreadId($v1, $raw = false)
    {
        $this->setFieldValue('thread_id', $v1, $raw);
    }


    public function getParentId()
    {
        return $this->getFieldValue('parent_id');
    }

    public function setParentId($v1, $raw = false)
    {
        $this->setFieldValue('parent_id', $v1, $raw);
    }


    public function getUserId()
    {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw = false)
    {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getUserString()
    {
        return $this->getFieldValue('user_string');
    }

    public function setUserString($v1, $raw = false)
    {
        $this->setFieldValue('user_string', $v1, $raw);
    }


    public function getTitle()
    {
        return $this->getFieldValue('title');
    }

    public function setTitle($v1, $raw = false)
    {
        $this->setFieldValue('title', $v1, $raw);
    }


    public function getText()
    {
        return $this->getFieldValue('text');
    }

    public function setText($v1, $raw = false)
    {
        $this->setFieldValue('text', $v1, $raw);
    }


    public function getDatePosted()
    {
        return $this->getFieldValue('date_posted');
    }

    public function setDatePosted($v1, $raw = false)
    {
        $this->setFieldValue('date_posted', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getRevisionNumber()
    {
        return $this->getFieldValue('revision_number');
    }

    public function setRevisionNumber($v1, $raw = false)
    {
        $this->setFieldValue('revision_number', $v1, $raw);
    }


    public function getRevisionId()
    {
        return $this->getFieldValue('revision_id');
    }

    public function setRevisionId($v1, $raw = false)
    {
        $this->setFieldValue('revision_id', $v1, $raw);
    }


    public function getDateLastEdited()
    {
        return $this->getFieldValue('date_last_edited');
    }

    public function setDateLastEdited($v1, $raw = false)
    {
        $this->setFieldValue('date_last_edited', $v1, $raw);
    }


    public function getEditedUserId()
    {
        return $this->getFieldValue('edited_user_id');
    }

    public function setEditedUserId($v1, $raw = false)
    {
        $this->setFieldValue('edited_user_id', $v1, $raw);
    }


    public function getEditedUserString()
    {
        return $this->getFieldValue('edited_user_string');
    }

    public function setEditedUserString($v1, $raw = false)
    {
        $this->setFieldValue('edited_user_string', $v1, $raw);
    }
}
