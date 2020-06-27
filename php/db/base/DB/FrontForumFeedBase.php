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
 * Base class mapped to the database table front_forum_feed.
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
