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
 * Base peer class mapped to the database table forum_category.
 */
class ForumCategoryPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='forum_category';
        $this->objectName='DB\\ForumCategory';
        $this->primaryKeyName = 'category_id';
        $this->fieldNames = array( 'category_id' ,  'group_id' ,  'name' ,  'description' ,  'number_posts' ,  'number_threads' ,  'last_post_id' ,  'permissions_default' ,  'permissions' ,  'max_nest_level' ,  'sort_index' ,  'site_id' ,  'per_page_discussion' );
        $this->fieldTypes = array( 'category_id' => 'serial',  'group_id' => 'int',  'name' => 'varchar(80)',  'description' => 'text',  'number_posts' => 'int',  'number_threads' => 'int',  'last_post_id' => 'int',  'permissions_default' => 'boolean',  'permissions' => 'varchar(200)',  'max_nest_level' => 'int',  'sort_index' => 'int',  'site_id' => 'int',  'per_page_discussion' => 'boolean');
        $this->defaultValues = array( 'number_posts' => '0',  'number_threads' => '0',  'permissions_default' => 'true',  'sort_index' => '0',  'per_page_discussion' => 'false');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\ForumCategoryPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
