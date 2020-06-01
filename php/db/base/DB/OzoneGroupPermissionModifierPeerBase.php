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
 * Base peer class mapped to the database table ozone_group_permission_modifier.
 */
class OzoneGroupPermissionModifierPeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='ozone_group_permission_modifier';
		$this->objectName='DB\\OzoneGroupPermissionModifier';
		$this->primaryKeyName = 'group_permission_id';
		$this->fieldNames = array( 'group_permission_id' ,  'group_id' ,  'permission_id' ,  'modifier' );
		$this->fieldTypes = array( 'group_permission_id' => 'serial',  'group_id' => 'varchar(20)',  'permission_id' => 'varchar(20)',  'modifier' => 'int');
		$this->defaultValues = array();
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB\\OzoneGroupPermissionModifierPeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}