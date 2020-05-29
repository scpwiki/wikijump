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
 * Base class mapped to the database table private_message.
 */
class DB_PrivateMessageBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='private_message';
		$this->peerName = 'DB_PrivateMessagePeer';
		$this->primaryKeyName = 'message_id';
		$this->fieldNames = array( 'message_id' ,  'from_user_id' ,  'to_user_id' ,  'subject' ,  'body' ,  'date' ,  'flag' ,  'flag_new' );
		
		//$this->fieldDefaultValues=
	}


	
		
	
		
	public function getMessageId() {
		return $this->getFieldValue('message_id');
	}
	
	public function setMessageId($v1, $raw=false) {
		$this->setFieldValue('message_id', $v1, $raw); 
	}
	
		
	public function getFromUserId() {
		return $this->getFieldValue('from_user_id');
	}
	
	public function setFromUserId($v1, $raw=false) {
		$this->setFieldValue('from_user_id', $v1, $raw); 
	}
	
		
	public function getToUserId() {
		return $this->getFieldValue('to_user_id');
	}
	
	public function setToUserId($v1, $raw=false) {
		$this->setFieldValue('to_user_id', $v1, $raw); 
	}
	
		
	public function getSubject() {
		return $this->getFieldValue('subject');
	}
	
	public function setSubject($v1, $raw=false) {
		$this->setFieldValue('subject', $v1, $raw); 
	}
	
		
	public function getBody() {
		return $this->getFieldValue('body');
	}
	
	public function setBody($v1, $raw=false) {
		$this->setFieldValue('body', $v1, $raw); 
	}
	
		
	public function getDate() {
		return $this->getFieldValue('date');
	}
	
	public function setDate($v1, $raw=false) {
		$this->setFieldValue('date', $v1, $raw); 
	}
	
		
	public function getFlag() {
		return $this->getFieldValue('flag');
	}
	
	public function setFlag($v1, $raw=false) {
		$this->setFieldValue('flag', $v1, $raw); 
	}
	
		
	public function getFlagNew() {
		return $this->getFieldValue('flag_new');
	}
	
	public function setFlagNew($v1, $raw=false) {
		$this->setFieldValue('flag_new', $v1, $raw); 
	}
	
		
	

}
