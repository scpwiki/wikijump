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
 * Base class mapped to the database table storage_item.
 */
class StorageItemBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='storage_item';
		$this->peerName = 'DB\\StorageItemPeer';
		$this->primaryKeyName = 'item_id';
		$this->fieldNames = array( 'item_id' ,  'date' ,  'timeout' ,  'data' );
		
		//$this->fieldDefaultValues=
	}


	
		
	
		
	public function getItemId() {
		return $this->getFieldValue('item_id');
	}
	
	public function setItemId($v1, $raw=false) {
		$this->setFieldValue('item_id', $v1, $raw); 
	}
	
		
	public function getDate() {
		return $this->getFieldValue('date');
	}
	
	public function setDate($v1, $raw=false) {
		$this->setFieldValue('date', $v1, $raw); 
	}
	
		
	public function getTimeout() {
		return $this->getFieldValue('timeout');
	}
	
	public function setTimeout($v1, $raw=false) {
		$this->setFieldValue('timeout', $v1, $raw); 
	}
	
		
	public function getData() {
		return $this->getFieldValue('data');
	}
	
	public function setData($v1, $raw=false) {
		$this->setFieldValue('data', $v1, $raw); 
	}
	
		
	

}
