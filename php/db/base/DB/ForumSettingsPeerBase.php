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
 * Base peer class mapped to the database table forum_settings.
 */
class ForumSettingsPeerBase extends BaseDBPeer {
	public static $peerInstance;

	protected function internalInit(){
		$this->tableName='forum_settings';
		$this->objectName='DB\\ForumSettings';
		$this->primaryKeyName = 'site_id';
		$this->fieldNames = array( 'site_id' ,  'permissions' ,  'per_page_discussion' ,  'max_nest_level' );
		$this->fieldTypes = array( 'site_id' => 'int',  'permissions' => 'varchar(200)',  'per_page_discussion' => 'boolean',  'max_nest_level' => 'int');
		$this->defaultValues = array( 'per_page_discussion' => 'false',  'max_nest_level' => '0');
	}

	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB\\ForumSettingsPeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}