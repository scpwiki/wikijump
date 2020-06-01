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
 * Base class mapped to the database table forum_group.
 */
class ForumGroupBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='forum_group';
		$this->peerName = 'DB\\ForumGroupPeer';
		$this->primaryKeyName = 'group_id';
		$this->fieldNames = array( 'group_id' ,  'name' ,  'description' ,  'sort_index' ,  'site_id' ,  'visible' );
		
		//$this->fieldDefaultValues=
	}


	
		
	
		
	public function getGroupId() {
		return $this->getFieldValue('group_id');
	}
	
	public function setGroupId($v1, $raw=false) {
		$this->setFieldValue('group_id', $v1, $raw); 
	}
	
		
	public function getName() {
		return $this->getFieldValue('name');
	}
	
	public function setName($v1, $raw=false) {
		$this->setFieldValue('name', $v1, $raw); 
	}
	
		
	public function getDescription() {
		return $this->getFieldValue('description');
	}
	
	public function setDescription($v1, $raw=false) {
		$this->setFieldValue('description', $v1, $raw); 
	}
	
		
	public function getSortIndex() {
		return $this->getFieldValue('sort_index');
	}
	
	public function setSortIndex($v1, $raw=false) {
		$this->setFieldValue('sort_index', $v1, $raw); 
	}
	
		
	public function getSiteId() {
		return $this->getFieldValue('site_id');
	}
	
	public function setSiteId($v1, $raw=false) {
		$this->setFieldValue('site_id', $v1, $raw); 
	}
	
		
	public function getVisible() {
		return $this->getFieldValue('visible');
	}
	
	public function setVisible($v1, $raw=false) {
		$this->setFieldValue('visible', $v1, $raw); 
	}
	
		
	

}
