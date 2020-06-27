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
 * Base class mapped to the database table site_backup.
 */
class SiteBackupBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='site_backup';
        $this->peerName = 'DB\\SiteBackupPeer';
        $this->primaryKeyName = 'backup_id';
        $this->fieldNames = array( 'backup_id' ,  'site_id' ,  'status' ,  'backup_source' ,  'backup_files' ,  'date' ,  'rand' );

        //$this->fieldDefaultValues=
    }






    public function getBackupId()
    {
        return $this->getFieldValue('backup_id');
    }

    public function setBackupId($v1, $raw = false)
    {
        $this->setFieldValue('backup_id', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getStatus()
    {
        return $this->getFieldValue('status');
    }

    public function setStatus($v1, $raw = false)
    {
        $this->setFieldValue('status', $v1, $raw);
    }


    public function getBackupSource()
    {
        return $this->getFieldValue('backup_source');
    }

    public function setBackupSource($v1, $raw = false)
    {
        $this->setFieldValue('backup_source', $v1, $raw);
    }


    public function getBackupFiles()
    {
        return $this->getFieldValue('backup_files');
    }

    public function setBackupFiles($v1, $raw = false)
    {
        $this->setFieldValue('backup_files', $v1, $raw);
    }


    public function getDate()
    {
        return $this->getFieldValue('date');
    }

    public function setDate($v1, $raw = false)
    {
        $this->setFieldValue('date', $v1, $raw);
    }


    public function getRand()
    {
        return $this->getFieldValue('rand');
    }

    public function setRand($v1, $raw = false)
    {
        $this->setFieldValue('rand', $v1, $raw);
    }
}
