<?php
/**
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008-2020, Wikidot Inc., SCP Wiki Technical Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 *
 * @category Wikidot
 * @package Wikidot
 * @version \$Id\$
 * @copyright Copyright (c) 2008-2020, Wikidot Inc., SCP Wiki Technical Team
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table forum_category.
 */
class ForumCategoryBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='forum_category';
        $this->peerName = 'DB\\ForumCategoryPeer';
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
