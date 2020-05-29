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
 * Base class mapped to the database table page_inclusion.
 */
class PageInclusionBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='page_inclusion';
		$this->peerName = 'DB_PageInclusionPeer';
		$this->primaryKeyName = 'inclusion_id';
		$this->fieldNames = array( 'inclusion_id' ,  'site_id' ,  'including_page_id' ,  'included_page_id' ,  'included_page_name' );
		
		//$this->fieldDefaultValues=
	}


	
		
	
		
	public function getInclusionId() {
		return $this->getFieldValue('inclusion_id');
	}
	
	public function setInclusionId($v1, $raw=false) {
		$this->setFieldValue('inclusion_id', $v1, $raw); 
	}
	
		
	public function getSiteId() {
		return $this->getFieldValue('site_id');
	}
	
	public function setSiteId($v1, $raw=false) {
		$this->setFieldValue('site_id', $v1, $raw); 
	}
	
		
	public function getIncludingPageId() {
		return $this->getFieldValue('including_page_id');
	}
	
	public function setIncludingPageId($v1, $raw=false) {
		$this->setFieldValue('including_page_id', $v1, $raw); 
	}
	
		
	public function getIncludedPageId() {
		return $this->getFieldValue('included_page_id');
	}
	
	public function setIncludedPageId($v1, $raw=false) {
		$this->setFieldValue('included_page_id', $v1, $raw); 
	}
	
		
	public function getIncludedPageName() {
		return $this->getFieldValue('included_page_name');
	}
	
	public function setIncludedPageName($v1, $raw=false) {
		$this->setFieldValue('included_page_name', $v1, $raw); 
	}
	
		
	

}
