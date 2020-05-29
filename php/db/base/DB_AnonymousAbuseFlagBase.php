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
 * Base class mapped to the database table anonymous_abuse_flag.
 */
class DB_AnonymousAbuseFlagBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='anonymous_abuse_flag';
		$this->peerName = 'DB_AnonymousAbuseFlagPeer';
		$this->primaryKeyName = 'flag_id';
		$this->fieldNames = array( 'flag_id' ,  'user_id' ,  'address' ,  'proxy' ,  'site_id' ,  'site_valid' ,  'global_valid' );
		
		//$this->fieldDefaultValues=
	}


	
		
	
		
	public function getFlagId() {
		return $this->getFieldValue('flag_id');
	}
	
	public function setFlagId($v1, $raw=false) {
		$this->setFieldValue('flag_id', $v1, $raw); 
	}
	
		
	public function getUserId() {
		return $this->getFieldValue('user_id');
	}
	
	public function setUserId($v1, $raw=false) {
		$this->setFieldValue('user_id', $v1, $raw); 
	}
	
		
	public function getAddress() {
		return $this->getFieldValue('address');
	}
	
	public function setAddress($v1, $raw=false) {
		$this->setFieldValue('address', $v1, $raw); 
	}
	
		
	public function getProxy() {
		return $this->getFieldValue('proxy');
	}
	
	public function setProxy($v1, $raw=false) {
		$this->setFieldValue('proxy', $v1, $raw); 
	}
	
		
	public function getSiteId() {
		return $this->getFieldValue('site_id');
	}
	
	public function setSiteId($v1, $raw=false) {
		$this->setFieldValue('site_id', $v1, $raw); 
	}
	
		
	public function getSiteValid() {
		return $this->getFieldValue('site_valid');
	}
	
	public function setSiteValid($v1, $raw=false) {
		$this->setFieldValue('site_valid', $v1, $raw); 
	}
	
		
	public function getGlobalValid() {
		return $this->getFieldValue('global_valid');
	}
	
	public function setGlobalValid($v1, $raw=false) {
		$this->setFieldValue('global_valid', $v1, $raw); 
	}
	
		
	

}
