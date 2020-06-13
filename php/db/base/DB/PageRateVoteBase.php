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
 * Base class mapped to the database table page_rate_vote.
 */
class PageRateVoteBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='page_rate_vote';
        $this->peerName = 'DB\\PageRateVotePeer';
        $this->primaryKeyName = 'rate_id';
        $this->fieldNames = array( 'rate_id' ,  'user_id' ,  'page_id' ,  'rate' ,  'date' );

        //$this->fieldDefaultValues=
    }






    public function getRateId()
    {
        return $this->getFieldValue('rate_id');
    }

    public function setRateId($v1, $raw = false)
    {
        $this->setFieldValue('rate_id', $v1, $raw);
    }


    public function getUserId()
    {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw = false)
    {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getPageId()
    {
        return $this->getFieldValue('page_id');
    }

    public function setPageId($v1, $raw = false)
    {
        $this->setFieldValue('page_id', $v1, $raw);
    }


    public function getRate()
    {
        return $this->getFieldValue('rate');
    }

    public function setRate($v1, $raw = false)
    {
        $this->setFieldValue('rate', $v1, $raw);
    }


    public function getDate()
    {
        return $this->getFieldValue('date');
    }

    public function setDate($v1, $raw = false)
    {
        $this->setFieldValue('date', $v1, $raw);
    }
}
