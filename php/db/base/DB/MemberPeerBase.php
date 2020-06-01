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
 * Base peer class mapped to the database table member.
 */
class MemberPeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='member';
		$this->objectName='DB\\Member';
		$this->primaryKeyName = 'member_id';
		$this->fieldNames = array( 'member_id' ,  'site_id' ,  'user_id' ,  'date_joined' ,  'allow_newsletter' );
		$this->fieldTypes = array( 'member_id' => 'serial',  'site_id' => 'int',  'user_id' => 'int',  'date_joined' => 'timestamp',  'allow_newsletter' => 'boolean');
		$this->defaultValues = array( 'allow_newsletter' => 'true');
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB\\MemberPeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}