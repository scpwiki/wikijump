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
 * Base class mapped to the database table forum_settings.
 */
class ForumSettingsBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='forum_settings';
        $this->peerName = 'DB\\ForumSettingsPeer';
        $this->primaryKeyName = 'site_id';
        $this->fieldNames = array( 'site_id' ,  'permissions' ,  'per_page_discussion' ,  'max_nest_level' );

        //$this->fieldDefaultValues=
    }






    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getPermissions()
    {
        return $this->getFieldValue('permissions');
    }

    public function setPermissions($v1, $raw = false)
    {
        $this->setFieldValue('permissions', $v1, $raw);
    }


    public function getPerPageDiscussion()
    {
        return $this->getFieldValue('per_page_discussion');
    }

    public function setPerPageDiscussion($v1, $raw = false)
    {
        $this->setFieldValue('per_page_discussion', $v1, $raw);
    }


    public function getMaxNestLevel()
    {
        return $this->getFieldValue('max_nest_level');
    }

    public function setMaxNestLevel($v1, $raw = false)
    {
        $this->setFieldValue('max_nest_level', $v1, $raw);
    }
}
