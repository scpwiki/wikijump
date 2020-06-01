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
 * Base class mapped to the database table page_tag.
 */
class PageTagBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='page_tag';
		$this->peerName = 'DB\\PageTagPeer';
		$this->primaryKeyName = 'tag_id';
		$this->fieldNames = array( 'tag_id' ,  'site_id' ,  'page_id' ,  'tag' );
		
		//$this->fieldDefaultValues=
	}


	
		
	
		
	public function getTagId() {
		return $this->getFieldValue('tag_id');
	}
	
	public function setTagId($v1, $raw=false) {
		$this->setFieldValue('tag_id', $v1, $raw); 
	}
	
		
	public function getSiteId() {
		return $this->getFieldValue('site_id');
	}
	
	public function setSiteId($v1, $raw=false) {
		$this->setFieldValue('site_id', $v1, $raw); 
	}
	
		
	public function getPageId() {
		return $this->getFieldValue('page_id');
	}
	
	public function setPageId($v1, $raw=false) {
		$this->setFieldValue('page_id', $v1, $raw); 
	}
	
		
	public function getTag() {
		return $this->getFieldValue('tag');
	}
	
	public function setTag($v1, $raw=false) {
		$this->setFieldValue('tag', $v1, $raw); 
	}
	
		
	

}
