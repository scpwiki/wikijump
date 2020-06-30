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
 * Base class mapped to the database table user_karma.
 */
class UserKarmaBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='user_karma';
        $this->peerName = 'DB\\UserKarmaPeer';
        $this->primaryKeyName = 'user_id';
        $this->fieldNames = array( 'user_id' ,  'points' ,  'level' );

        //$this->fieldDefaultValues=
    }






    public function getUserId()
    {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw = false)
    {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getPoints()
    {
        return $this->getFieldValue('points');
    }

    public function setPoints($v1, $raw = false)
    {
        $this->setFieldValue('points', $v1, $raw);
    }


    public function getLevel()
    {
        return $this->getFieldValue('level');
    }

    public function setLevel($v1, $raw = false)
    {
        $this->setFieldValue('level', $v1, $raw);
    }
}
