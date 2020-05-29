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
 * Base class mapped to the database table ozone_user_group_relation.
 */
class DB_OzoneUserGroupRelationBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='ozone_user_group_relation';
		$this->peerName = 'DB_OzoneUserGroupRelationPeer';
		$this->primaryKeyName = 'user_group_id';
		$this->fieldNames = array( 'user_group_id' ,  'user_id' ,  'group_id' );
		
		//$this->fieldDefaultValues=
	}


	
			public function getOzoneUser(){
			if(is_array($this->prefetched)){
			if(in_array('ozone_user', $this->prefetched)){
				if(in_array('ozone_user', $this->prefetchedObjects)){
					return $this->prefetchedObjects['ozone_user'];
				} else {
					
					$obj = new DB_OzoneUser($this->sourceRow);
					$obj->setNew(false);
					//$obj->prefetched = $this->prefetched;
					//$obj->sourceRow = $this->sourceRow;
					$this->prefetchedObjects['ozone_user'] = $obj;
					return $obj;
				}
			}
		}
		$foreignPeerClassName = 'DB_OzoneUserPeer';	
		$fpeer = new $foreignPeerClassName();
		
		$criteria = new Criteria();
		
		$criteria->add("user_id", $this->fieldValues['user_id']);
		
		$result = $fpeer->selectOneByCriteria($criteria);
		return $result;
	}
	
		public function setOzoneUser($primaryObject){
			$this->fieldValues['user_id'] = $primaryObject->getFieldValue('user_id');
	}
			public function getOzoneGroup(){
			if(is_array($this->prefetched)){
			if(in_array('ozone_group', $this->prefetched)){
				if(in_array('ozone_group', $this->prefetchedObjects)){
					return $this->prefetchedObjects['ozone_group'];
				} else {
					
					$obj = new DB_OzoneGroup($this->sourceRow);
					$obj->setNew(false);
					//$obj->prefetched = $this->prefetched;
					//$obj->sourceRow = $this->sourceRow;
					$this->prefetchedObjects['ozone_group'] = $obj;
					return $obj;
				}
			}
		}
		$foreignPeerClassName = 'DB_OzoneGroupPeer';	
		$fpeer = new $foreignPeerClassName();
		
		$criteria = new Criteria();
		
		$criteria->add("group_id", $this->fieldValues['group_id']);
		
		$result = $fpeer->selectOneByCriteria($criteria);
		return $result;
	}
	
		public function setOzoneGroup($primaryObject){
			$this->fieldValues['group_id'] = $primaryObject->getFieldValue('group_id');
	}
		
	
		
	public function getUserGroupId() {
		return $this->getFieldValue('user_group_id');
	}
	
	public function setUserGroupId($v1, $raw=false) {
		$this->setFieldValue('user_group_id', $v1, $raw); 
	}
	
		
	public function getUserId() {
		return $this->getFieldValue('user_id');
	}
	
	public function setUserId($v1, $raw=false) {
		$this->setFieldValue('user_id', $v1, $raw); 
	}
	
		
	public function getGroupId() {
		return $this->getFieldValue('group_id');
	}
	
	public function setGroupId($v1, $raw=false) {
		$this->setFieldValue('group_id', $v1, $raw); 
	}
	
		
	

}
