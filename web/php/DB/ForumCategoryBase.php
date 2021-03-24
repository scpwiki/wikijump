<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table forum_category.
 */
class ForumCategoryBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='forum_category';
        $this->peerName = 'Wikidot\\DB\\ForumCategoryPeer';
        $this->primaryKeyName = 'category_id';
        $this->fieldNames = array( 'category_id' ,  'group_id' ,  'name' ,  'description' ,  'number_posts' ,  'number_threads' ,  'last_post_id' ,  'permissions_default' ,  'permissions' ,  'max_nest_level' ,  'sort_index' ,  'site_id' ,  'per_page_discussion' );

        //$this->fieldDefaultValues=
    }






    public function getCategoryId()
    {
        return $this->getFieldValue('category_id');
    }

    public function setCategoryId($v1, $raw = false)
    {
        $this->setFieldValue('category_id', $v1, $raw);
    }


    public function getGroupId()
    {
        return $this->getFieldValue('group_id');
    }

    public function setGroupId($v1, $raw = false)
    {
        $this->setFieldValue('group_id', $v1, $raw);
    }


    public function getName()
    {
        return $this->getFieldValue('name');
    }

    public function setName($v1, $raw = false)
    {
        $this->setFieldValue('name', $v1, $raw);
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


    public function getNumberThreads()
    {
        return $this->getFieldValue('number_threads');
    }

    public function setNumberThreads($v1, $raw = false)
    {
        $this->setFieldValue('number_threads', $v1, $raw);
    }


    public function getLastPostId()
    {
        return $this->getFieldValue('last_post_id');
    }

    public function setLastPostId($v1, $raw = false)
    {
        $this->setFieldValue('last_post_id', $v1, $raw);
    }


    public function getPermissionsDefault()
    {
        return $this->getFieldValue('permissions_default');
    }

    public function setPermissionsDefault($v1, $raw = false)
    {
        $this->setFieldValue('permissions_default', $v1, $raw);
    }


    public function getPermissions()
    {
        return $this->getFieldValue('permissions');
    }

    public function setPermissions($v1, $raw = false)
    {
        $this->setFieldValue('permissions', $v1, $raw);
    }


    public function getMaxNestLevel()
    {
        return $this->getFieldValue('max_nest_level');
    }

    public function setMaxNestLevel($v1, $raw = false)
    {
        $this->setFieldValue('max_nest_level', $v1, $raw);
    }


    public function getSortIndex()
    {
        return $this->getFieldValue('sort_index');
    }

    public function setSortIndex($v1, $raw = false)
    {
        $this->setFieldValue('sort_index', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getPerPageDiscussion()
    {
        return $this->getFieldValue('per_page_discussion');
    }

    public function setPerPageDiscussion($v1, $raw = false)
    {
        $this->setFieldValue('per_page_discussion', $v1, $raw);
    }
}
