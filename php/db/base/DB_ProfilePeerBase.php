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
 
/**
 * Base peer class mapped to the database table profile.
 */
class DB_ProfilePeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='profile';
		$this->objectName='DB_Profile';
		$this->primaryKeyName = 'user_id';
		$this->fieldNames = array( 'user_id' ,  'real_name' ,  'gender' ,  'birthday_day' ,  'birthday_month' ,  'birthday_year' ,  'about' ,  'location' ,  'website' ,  'im_aim' ,  'im_gadu_gadu' ,  'im_google_talk' ,  'im_icq' ,  'im_jabber' ,  'im_msn' ,  'im_yahoo' ,  'change_screen_name_count' );
		$this->fieldTypes = array( 'user_id' => 'int',  'real_name' => 'varchar(70)',  'gender' => 'char(1)',  'birthday_day' => 'int',  'birthday_month' => 'int',  'birthday_year' => 'int',  'about' => 'text',  'location' => 'varchar(70)',  'website' => 'varchar(100)',  'im_aim' => 'varchar(100)',  'im_gadu_gadu' => 'varchar(100)',  'im_google_talk' => 'varchar(100)',  'im_icq' => 'varchar(100)',  'im_jabber' => 'varchar(100)',  'im_msn' => 'varchar(100)',  'im_yahoo' => 'varchar(100)',  'change_screen_name_count' => 'int');
		$this->defaultValues = array( 'change_screen_name_count' => '0');
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB_ProfilePeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}