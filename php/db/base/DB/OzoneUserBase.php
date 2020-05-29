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



 
/**
 * Base class mapped to the database table ozone_user.
 */
class OzoneUserBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='ozone_user';
		$this->peerName = 'DB_OzoneUserPeer';
		$this->primaryKeyName = 'user_id';
		$this->fieldNames = array( 'user_id' ,  'name' ,  'nick_name' ,  'password' ,  'email' ,  'unix_name' ,  'last_login' ,  'registered_date' ,  'super_admin' ,  'super_moderator' ,  'language' );
		
		//$this->fieldDefaultValues=
	}


	
		
	
		
	public function getUserId() {
		return $this->getFieldValue('user_id');
	}
	
	public function setUserId($v1, $raw=false) {
		$this->setFieldValue('user_id', $v1, $raw); 
	}
	
		
	public function getName() {
		return $this->getFieldValue('name');
	}
	
	public function setName($v1, $raw=false) {
		$this->setFieldValue('name', $v1, $raw); 
	}
	
		
	public function getNickName() {
		return $this->getFieldValue('nick_name');
	}
	
	public function setNickName($v1, $raw=false) {
		$this->setFieldValue('nick_name', $v1, $raw); 
	}
	
		
	public function getPassword() {
		return $this->getFieldValue('password');
	}
	
	public function setPassword($v1, $raw=false) {
		$this->setFieldValue('password', $v1, $raw); 
	}
	
		
	public function getEmail() {
		return $this->getFieldValue('email');
	}
	
	public function setEmail($v1, $raw=false) {
		$this->setFieldValue('email', $v1, $raw); 
	}
	
		
	public function getUnixName() {
		return $this->getFieldValue('unix_name');
	}
	
	public function setUnixName($v1, $raw=false) {
		$this->setFieldValue('unix_name', $v1, $raw); 
	}
	
		
	public function getLastLogin() {
		return $this->getFieldValue('last_login');
	}
	
	public function setLastLogin($v1, $raw=false) {
		$this->setFieldValue('last_login', $v1, $raw); 
	}
	
		
	public function getRegisteredDate() {
		return $this->getFieldValue('registered_date');
	}
	
	public function setRegisteredDate($v1, $raw=false) {
		$this->setFieldValue('registered_date', $v1, $raw); 
	}
	
		
	public function getSuperAdmin() {
		return $this->getFieldValue('super_admin');
	}
	
	public function setSuperAdmin($v1, $raw=false) {
		$this->setFieldValue('super_admin', $v1, $raw); 
	}
	
		
	public function getSuperModerator() {
		return $this->getFieldValue('super_moderator');
	}
	
	public function setSuperModerator($v1, $raw=false) {
		$this->setFieldValue('super_moderator', $v1, $raw); 
	}
	
		
	public function getLanguage() {
		return $this->getFieldValue('language');
	}
	
	public function setLanguage($v1, $raw=false) {
		$this->setFieldValue('language', $v1, $raw); 
	}
	
		
	

}
