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
 * Base class mapped to the database table page_source.
 */
class PageSourceBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='page_source';
        $this->peerName = 'DB\\PageSourcePeer';
        $this->primaryKeyName = 'source_id';
        $this->fieldNames = array( 'source_id' ,  'text' );

        //$this->fieldDefaultValues=
    }






    public function getSourceId()
    {
        return $this->getFieldValue('source_id');
    }

    public function setSourceId($v1, $raw = false)
    {
        $this->setFieldValue('source_id', $v1, $raw);
    }


    public function getText()
    {
        return $this->getFieldValue('text');
    }

    public function setText($v1, $raw = false)
    {
        $this->setFieldValue('text', $v1, $raw);
    }
}
