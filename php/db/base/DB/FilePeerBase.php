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
 * Base peer class mapped to the database table file.
 */
class FilePeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='file';
		$this->objectName='DB\\File';
		$this->primaryKeyName = 'file_id';
		$this->fieldNames = array( 'file_id' ,  'page_id' ,  'site_id' ,  'filename' ,  'mimetype' ,  'description' ,  'description_short' ,  'comment' ,  'size' ,  'date_added' ,  'user_id' ,  'user_string' ,  'has_resized' );
		$this->fieldTypes = array( 'file_id' => 'serial',  'page_id' => 'int',  'site_id' => 'int',  'filename' => 'varchar(100)',  'mimetype' => 'varchar(100)',  'description' => 'varchar(200)',  'description_short' => 'varchar(200)',  'comment' => 'varchar(400)',  'size' => 'int',  'date_added' => 'timestamp',  'user_id' => 'int',  'user_string' => 'varchar(80)',  'has_resized' => 'boolean');
		$this->defaultValues = array( 'has_resized' => 'false');
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB\\FilePeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}