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
 * Base peer class mapped to the database table ucookie.
 */
class DB_UcookiePeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='ucookie';
		$this->objectName='DB_Ucookie';
		$this->primaryKeyName = 'ucookie_id';
		$this->fieldNames = array( 'ucookie_id' ,  'site_id' ,  'session_id' ,  'date_granted' );
		$this->fieldTypes = array( 'ucookie_id' => 'varchar(100)',  'site_id' => 'int',  'session_id' => 'varchar(60)',  'date_granted' => 'timestamp');
		$this->defaultValues = array();
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB_UcookiePeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}