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

use BaseDBObject;
use DB\OzoneGroup;
use Criteria;
use DB\OzonePermission;



 
/**
 * Base class mapped to the database table ozone_group_permission_modifier.
 */
class OzoneGroupPermissionModifierBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='ozone_group_permission_modifier';
		$this->peerName = 'DB\\OzoneGroupPermissionModifierPeer';
		$this->primaryKeyName = 'group_permission_id';
		$this->fieldNames = array( 'group_permission_id' ,  'group_id' ,  'permission_id' ,  'modifier' );
		
		//$this->fieldDefaultValues=
	}


	
			public function getOzoneGroup(){
			if(is_array($this->prefetched)){
			if(in_array('ozone_group', $this->prefetched)){
				if(in_array('ozone_group', $this->prefetchedObjects)){
					return $this->prefetchedObjects['ozone_group'];
				} else {
					
					$obj = new OzoneGroup($this->sourceRow);
					$obj->setNew(false);
					//$obj->prefetched = $this->prefetched;
					//$obj->sourceRow = $this->sourceRow;
					$this->prefetchedObjects['ozone_group'] = $obj;
					return $obj;
				}
			}
		}
		$foreignPeerClassName = 'DB\\OzoneGroupPeer';	
		$fpeer = new $foreignPeerClassName();
		
		$criteria = new Criteria();
		
		$criteria->add("group_id", $this->fieldValues['group_id']);
		
		$result = $fpeer->selectOneByCriteria($criteria);
		return $result;
	}
	
		public function setOzoneGroup($primaryObject){
			$this->fieldValues['group_id'] = $primaryObject->getFieldValue('group_id');
	}
			public function getOzonePermission(){
			if(is_array($this->prefetched)){
			if(in_array('ozone_permission', $this->prefetched)){
				if(in_array('ozone_permission', $this->prefetchedObjects)){
					return $this->prefetchedObjects['ozone_permission'];
				} else {
					
					$obj = new OzonePermission($this->sourceRow);
					$obj->setNew(false);
					//$obj->prefetched = $this->prefetched;
					//$obj->sourceRow = $this->sourceRow;
					$this->prefetchedObjects['ozone_permission'] = $obj;
					return $obj;
				}
			}
		}
		$foreignPeerClassName = 'DB\\OzonePermissionPeer';	
		$fpeer = new $foreignPeerClassName();
		
		$criteria = new Criteria();
		
		$criteria->add("permission_id", $this->fieldValues['permission_id']);
		
		$result = $fpeer->selectOneByCriteria($criteria);
		return $result;
	}
	
		public function setOzonePermission($primaryObject){
			$this->fieldValues['permission_id'] = $primaryObject->getFieldValue('permission_id');
	}
		
	
		
	public function getGroupPermissionId() {
		return $this->getFieldValue('group_permission_id');
	}
	
	public function setGroupPermissionId($v1, $raw=false) {
		$this->setFieldValue('group_permission_id', $v1, $raw); 
	}
	
		
	public function getGroupId() {
		return $this->getFieldValue('group_id');
	}
	
	public function setGroupId($v1, $raw=false) {
		$this->setFieldValue('group_id', $v1, $raw); 
	}
	
		
	public function getPermissionId() {
		return $this->getFieldValue('permission_id');
	}
	
	public function setPermissionId($v1, $raw=false) {
		$this->setFieldValue('permission_id', $v1, $raw); 
	}
	
		
	public function getModifier() {
		return $this->getFieldValue('modifier');
	}
	
	public function setModifier($v1, $raw=false) {
		$this->setFieldValue('modifier', $v1, $raw); 
	}
	
		
	

}
