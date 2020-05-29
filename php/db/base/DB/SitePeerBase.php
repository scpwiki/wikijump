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
 * Base peer class mapped to the database table site.
 */
class SitePeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='site';
		$this->objectName='DB_Site';
		$this->primaryKeyName = 'site_id';
		$this->fieldNames = array( 'site_id' ,  'name' ,  'subtitle' ,  'unix_name' ,  'description' ,  'language' ,  'date_created' ,  'custom_domain' ,  'default_page' ,  'visible' ,  'private' ,  'deleted' );
		$this->fieldTypes = array( 'site_id' => 'serial',  'name' => 'varchar(100)',  'subtitle' => 'varchar(60)',  'unix_name' => 'varchar(50)',  'description' => 'text',  'language' => 'varchar(10)',  'date_created' => 'timestamp',  'custom_domain' => 'varchar(60)',  'default_page' => 'varchar(80)',  'visible' => 'boolean',  'private' => 'boolean',  'deleted' => 'boolean');
		$this->defaultValues = array( 'language' => 'en',  'default_page' => 'start',  'visible' => 'true',  'private' => 'false',  'deleted' => 'false');
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB_SitePeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}