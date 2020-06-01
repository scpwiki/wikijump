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
 * Base peer class mapped to the database table ozone_session.
 */
class OzoneSessionPeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='ozone_session';
		$this->objectName='DB\\OzoneSession';
		$this->primaryKeyName = 'session_id';
		$this->fieldNames = array( 'session_id' ,  'started' ,  'last_accessed' ,  'ip_address' ,  'ip_address_ssl' ,  'ua_hash' ,  'check_ip' ,  'infinite' ,  'user_id' ,  'serialized_datablock' );
		$this->fieldTypes = array( 'session_id' => 'varchar(60)',  'started' => 'timestamp',  'last_accessed' => 'timestamp',  'ip_address' => 'varchar(90)',  'ip_address_ssl' => 'varchar(90)',  'ua_hash' => 'varchar(256)',  'check_ip' => 'boolean',  'infinite' => 'boolean',  'user_id' => 'int',  'serialized_datablock' => 'bytea');
		$this->defaultValues = array( 'check_ip' => 'false',  'infinite' => 'false');
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB\\OzoneSessionPeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}