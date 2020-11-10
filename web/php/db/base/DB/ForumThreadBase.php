<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table forum_thread.
 */
class ForumThreadBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='forum_thread';
        $this->peerName = 'DB\\ForumThreadPeer';
        $this->primaryKeyName = 'thread_id';
        $this->fieldNames = array( 'thread_id' ,  'user_id' ,  'user_string' ,  'category_id' ,  'title' ,  'description' ,  'number_posts' ,  'date_started' ,  'site_id' ,  'last_post_id' ,  'page_id' ,  'sticky' ,  'blocked' );

        //$this->fieldDefaultValues=
    }






    public function getThreadId()
    {
        return $this->getFieldValue('thread_id');
    }

    public function setThreadId($v1, $raw = false)
    {
        $this->setFieldValue('thread_id', $v1, $raw);
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


    public function getCategoryId()
    {
        return $this->getFieldValue('category_id');
    }

    public function setCategoryId($v1, $raw = false)
    {
        $this->setFieldValue('category_id', $v1, $raw);
    }


    public function getTitle()
    {
        return $this->getFieldValue('title');
    }

    public function setTitle($v1, $raw = false)
    {
        $this->setFieldValue('title', $v1, $raw);
    }


    public function getDescription()
    {
        return $this->getFieldValue('description');
    }

    public function setDescription($v1, $raw = false)
    {
        $this->setFieldValue('description', $v1, $raw);
    }


    public function getNumberPosts()
    {
        return $this->getFieldValue('number_posts');
    }

    public function setNumberPosts($v1, $raw = false)
    {
        $this->setFieldValue('number_posts', $v1, $raw);
    }


    public function getDateStarted()
    {
        return $this->getFieldValue('date_started');
    }

    public function setDateStarted($v1, $raw = false)
    {
        $this->setFieldValue('date_started', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getLastPostId()
    {
        return $this->getFieldValue('last_post_id');
    }

    public function setLastPostId($v1, $raw = false)
    {
        $this->setFieldValue('last_post_id', $v1, $raw);
    }


    public function getPageId()
    {
        return $this->getFieldValue('page_id');
    }

    public function setPageId($v1, $raw = false)
    {
        $this->setFieldValue('page_id', $v1, $raw);
    }


    public function getSticky()
    {
        return $this->getFieldValue('sticky');
    }

    public function setSticky($v1, $raw = false)
    {
        $this->setFieldValue('sticky', $v1, $raw);
    }


    public function getBlocked()
    {
        return $this->getFieldValue('blocked');
    }

    public function setBlocked($v1, $raw = false)
    {
        $this->setFieldValue('blocked', $v1, $raw);
    }
}
