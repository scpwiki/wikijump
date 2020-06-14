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

use BaseDBPeer;

/**
 * Base peer class mapped to the database table page_edit_lock.
 */
class PageEditLockPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='page_edit_lock';
        $this->objectName='DB\\PageEditLock';
        $this->primaryKeyName = 'lock_id';
        $this->fieldNames = array( 'lock_id' ,  'page_id' ,  'mode' ,  'section_id' ,  'range_start' ,  'range_end' ,  'page_unix_name' ,  'site_id' ,  'user_id' ,  'user_string' ,  'session_id' ,  'date_started' ,  'date_last_accessed' ,  'secret' );
        $this->fieldTypes = array( 'lock_id' => 'serial',  'page_id' => 'int',  'mode' => 'varchar(10)',  'section_id' => 'int',  'range_start' => 'int',  'range_end' => 'int',  'page_unix_name' => 'varchar(100)',  'site_id' => 'int',  'user_id' => 'int',  'user_string' => 'varchar(80)',  'session_id' => 'varchar(60)',  'date_started' => 'timestamp',  'date_last_accessed' => 'timestamp',  'secret' => 'varchar(100)');
        $this->defaultValues = array( 'mode' => 'page');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\PageEditLockPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
