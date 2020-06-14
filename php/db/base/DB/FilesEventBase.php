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
 * Base class mapped to the database table files_event.
 */
class FilesEventBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='files_event';
        $this->peerName = 'DB\\FilesEventPeer';
        $this->primaryKeyName = 'file_event_id';
        $this->fieldNames = array( 'file_event_id' ,  'filename' ,  'date' ,  'user_id' ,  'user_string' ,  'action' ,  'action_extra' );

        //$this->fieldDefaultValues=
    }






    public function getFileEventId()
    {
        return $this->getFieldValue('file_event_id');
    }

    public function setFileEventId($v1, $raw = false)
    {
        $this->setFieldValue('file_event_id', $v1, $raw);
    }


    public function getFilename()
    {
        return $this->getFieldValue('filename');
    }

    public function setFilename($v1, $raw = false)
    {
        $this->setFieldValue('filename', $v1, $raw);
    }


    public function getDate()
    {
        return $this->getFieldValue('date');
    }

    public function setDate($v1, $raw = false)
    {
        $this->setFieldValue('date', $v1, $raw);
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


    public function getAction()
    {
        return $this->getFieldValue('action');
    }

    public function setAction($v1, $raw = false)
    {
        $this->setFieldValue('action', $v1, $raw);
    }


    public function getActionExtra()
    {
        return $this->getFieldValue('action_extra');
    }

    public function setActionExtra($v1, $raw = false)
    {
        $this->setFieldValue('action_extra', $v1, $raw);
    }
}
