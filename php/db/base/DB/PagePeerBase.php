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

use BaseDBPeer;

/**
 * Base peer class mapped to the database table page.
 */
class PagePeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='page';
        $this->objectName='DB\\Page';
        $this->primaryKeyName = 'page_id';
        $this->fieldNames = array( 'page_id' ,  'site_id' ,  'category_id' ,  'parent_page_id' ,  'revision_id' ,  'source_id' ,  'metadata_id' ,  'revision_number' ,  'title' ,  'unix_name' ,  'date_created' ,  'date_last_edited' ,  'last_edit_user_id' ,  'last_edit_user_string' ,  'thread_id' ,  'owner_user_id' ,  'blocked' ,  'rate' );
        $this->fieldTypes = array( 'page_id' => 'serial',  'site_id' => 'int',  'category_id' => 'int',  'parent_page_id' => 'int',  'revision_id' => 'int',  'source_id' => 'int',  'metadata_id' => 'int',  'revision_number' => 'int',  'title' => 'varchar(256)',  'unix_name' => 'varchar(256)',  'date_created' => 'timestamp',  'date_last_edited' => 'timestamp',  'last_edit_user_id' => 'int',  'last_edit_user_string' => 'varchar(80)',  'thread_id' => 'int',  'owner_user_id' => 'int',  'blocked' => 'boolean',  'rate' => 'int');
        $this->defaultValues = array( 'revision_number' => '0',  'blocked' => 'false',  'rate' => '0');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            self::$peerInstance = new PagePeer();
        }
        return self::$peerInstance;
    }
}
