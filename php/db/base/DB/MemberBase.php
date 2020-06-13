<?php
/**
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
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
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table member.
 */
class MemberBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='member';
        $this->peerName = 'DB\\MemberPeer';
        $this->primaryKeyName = 'member_id';
        $this->fieldNames = array( 'member_id' ,  'site_id' ,  'user_id' ,  'date_joined' ,  'allow_newsletter' );

        //$this->fieldDefaultValues=
    }






    public function getMemberId()
    {
        return $this->getFieldValue('member_id');
    }

    public function setMemberId($v1, $raw = false)
    {
        $this->setFieldValue('member_id', $v1, $raw);
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


    public function getDateJoined()
    {
        return $this->getFieldValue('date_joined');
    }

    public function setDateJoined($v1, $raw = false)
    {
        $this->setFieldValue('date_joined', $v1, $raw);
    }


    public function getAllowNewsletter()
    {
        return $this->getFieldValue('allow_newsletter');
    }

    public function setAllowNewsletter($v1, $raw = false)
    {
        $this->setFieldValue('allow_newsletter', $v1, $raw);
    }
}
