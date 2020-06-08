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
 * Base peer class mapped to the database table ozone_user.
 */
class OzoneUserPeerBase extends BaseDBPeer {
	public static $peerInstance;

	protected function internalInit(){
		$this->tableName='ozone_user';
		$this->objectName='DB\\OzoneUser';
		$this->primaryKeyName = 'user_id';
		$this->fieldNames = array( 'user_id' ,  'name' ,  'nick_name' ,  'password' ,  'email' ,  'unix_name' ,  'last_login' ,  'registered_date' ,  'super_admin' ,  'super_moderator' ,  'language' );
		$this->fieldTypes = array( 'user_id' => 'serial',  'name' => 'varchar(99)',  'nick_name' => 'varchar(70)',  'password' => 'varchar(255)',  'email' => 'varchar(99)',  'unix_name' => 'varchar(99)',  'last_login' => 'timestamp',  'registered_date' => 'timestamp',  'super_admin' => 'boolean',  'super_moderator' => 'boolean',  'language' => 'varchar(10)');
		$this->defaultValues = array( 'super_admin' => 'false',  'super_moderator' => 'false',  'language' => 'en');
	}

	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB\\OzoneUserPeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}