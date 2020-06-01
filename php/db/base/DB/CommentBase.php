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
 * Base class mapped to the database table comment.
 */
class CommentBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='comment';
		$this->peerName = 'DB\\CommentPeer';
		$this->primaryKeyName = 'comment_id';
		$this->fieldNames = array( 'comment_id' ,  'page_id' ,  'parent_id' ,  'user_id' ,  'user_string' ,  'title' ,  'text' ,  'date_posted' ,  'site_id' ,  'revision_number' ,  'revision_id' ,  'date_last_edited' ,  'edited_user_id' ,  'edited_user_string' );
		
		//$this->fieldDefaultValues=
	}


	
		
	
		
	public function getCommentId() {
		return $this->getFieldValue('comment_id');
	}
	
	public function setCommentId($v1, $raw=false) {
		$this->setFieldValue('comment_id', $v1, $raw); 
	}
	
		
	public function getPageId() {
		return $this->getFieldValue('page_id');
	}
	
	public function setPageId($v1, $raw=false) {
		$this->setFieldValue('page_id', $v1, $raw); 
	}
	
		
	public function getParentId() {
		return $this->getFieldValue('parent_id');
	}
	
	public function setParentId($v1, $raw=false) {
		$this->setFieldValue('parent_id', $v1, $raw); 
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
	
		
	public function getTitle() {
		return $this->getFieldValue('title');
	}
	
	public function setTitle($v1, $raw=false) {
		$this->setFieldValue('title', $v1, $raw); 
	}
	
		
	public function getText() {
		return $this->getFieldValue('text');
	}
	
	public function setText($v1, $raw=false) {
		$this->setFieldValue('text', $v1, $raw); 
	}
	
		
	public function getDatePosted() {
		return $this->getFieldValue('date_posted');
	}
	
	public function setDatePosted($v1, $raw=false) {
		$this->setFieldValue('date_posted', $v1, $raw); 
	}
	
		
	public function getSiteId() {
		return $this->getFieldValue('site_id');
	}
	
	public function setSiteId($v1, $raw=false) {
		$this->setFieldValue('site_id', $v1, $raw); 
	}
	
		
	public function getRevisionNumber() {
		return $this->getFieldValue('revision_number');
	}
	
	public function setRevisionNumber($v1, $raw=false) {
		$this->setFieldValue('revision_number', $v1, $raw); 
	}
	
		
	public function getRevisionId() {
		return $this->getFieldValue('revision_id');
	}
	
	public function setRevisionId($v1, $raw=false) {
		$this->setFieldValue('revision_id', $v1, $raw); 
	}
	
		
	public function getDateLastEdited() {
		return $this->getFieldValue('date_last_edited');
	}
	
	public function setDateLastEdited($v1, $raw=false) {
		$this->setFieldValue('date_last_edited', $v1, $raw); 
	}
	
		
	public function getEditedUserId() {
		return $this->getFieldValue('edited_user_id');
	}
	
	public function setEditedUserId($v1, $raw=false) {
		$this->setFieldValue('edited_user_id', $v1, $raw); 
	}
	
		
	public function getEditedUserString() {
		return $this->getFieldValue('edited_user_string');
	}
	
	public function setEditedUserString($v1, $raw=false) {
		$this->setFieldValue('edited_user_string', $v1, $raw); 
	}
	
		
	

}
