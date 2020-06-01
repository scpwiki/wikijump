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
 * Base class mapped to the database table file.
 */
class FileBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='file';
		$this->peerName = 'DB\\FilePeer';
		$this->primaryKeyName = 'file_id';
		$this->fieldNames = array( 'file_id' ,  'page_id' ,  'site_id' ,  'filename' ,  'mimetype' ,  'description' ,  'description_short' ,  'comment' ,  'size' ,  'date_added' ,  'user_id' ,  'user_string' ,  'has_resized' );
		
		//$this->fieldDefaultValues=
	}


	
		
	
		
	public function getFileId() {
		return $this->getFieldValue('file_id');
	}
	
	public function setFileId($v1, $raw=false) {
		$this->setFieldValue('file_id', $v1, $raw); 
	}
	
		
	public function getPageId() {
		return $this->getFieldValue('page_id');
	}
	
	public function setPageId($v1, $raw=false) {
		$this->setFieldValue('page_id', $v1, $raw); 
	}
	
		
	public function getSiteId() {
		return $this->getFieldValue('site_id');
	}
	
	public function setSiteId($v1, $raw=false) {
		$this->setFieldValue('site_id', $v1, $raw); 
	}
	
		
	public function getFilename() {
		return $this->getFieldValue('filename');
	}
	
	public function setFilename($v1, $raw=false) {
		$this->setFieldValue('filename', $v1, $raw); 
	}
	
		
	public function getMimetype() {
		return $this->getFieldValue('mimetype');
	}
	
	public function setMimetype($v1, $raw=false) {
		$this->setFieldValue('mimetype', $v1, $raw); 
	}
	
		
	public function getDescription() {
		return $this->getFieldValue('description');
	}
	
	public function setDescription($v1, $raw=false) {
		$this->setFieldValue('description', $v1, $raw); 
	}
	
		
	public function getDescriptionShort() {
		return $this->getFieldValue('description_short');
	}
	
	public function setDescriptionShort($v1, $raw=false) {
		$this->setFieldValue('description_short', $v1, $raw); 
	}
	
		
	public function getComment() {
		return $this->getFieldValue('comment');
	}
	
	public function setComment($v1, $raw=false) {
		$this->setFieldValue('comment', $v1, $raw); 
	}
	
		
	public function getSize() {
		return $this->getFieldValue('size');
	}
	
	public function setSize($v1, $raw=false) {
		$this->setFieldValue('size', $v1, $raw); 
	}
	
		
	public function getDateAdded() {
		return $this->getFieldValue('date_added');
	}
	
	public function setDateAdded($v1, $raw=false) {
		$this->setFieldValue('date_added', $v1, $raw); 
	}
	
		
	public function getUserId() {
		return $this->getFieldValue('user_id');
	}
	
	public function setUserId($v1, $raw=false) {
		$this->setFieldValue('user_id', $v1, $raw); 
	}
	
		
	public function getUserString() {
		return $this->getFieldValue('user_string');
	}
	
	public function setUserString($v1, $raw=false) {
		$this->setFieldValue('user_string', $v1, $raw); 
	}
	
		
	public function getHasResized() {
		return $this->getFieldValue('has_resized');
	}
	
	public function setHasResized($v1, $raw=false) {
		$this->setFieldValue('has_resized', $v1, $raw); 
	}
	
		
	

}
