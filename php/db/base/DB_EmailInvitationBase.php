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
 * Base class mapped to the database table email_invitation.
 */
class DB_EmailInvitationBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='email_invitation';
		$this->peerName = 'DB_EmailInvitationPeer';
		$this->primaryKeyName = 'invitation_id';
		$this->fieldNames = array( 'invitation_id' ,  'hash' ,  'email' ,  'name' ,  'user_id' ,  'site_id' ,  'become_member' ,  'to_contacts' ,  'message' ,  'attempts' ,  'accepted' ,  'delivered' ,  'date' );
		
		//$this->fieldDefaultValues=
	}


	
		
	
		
	public function getInvitationId() {
		return $this->getFieldValue('invitation_id');
	}
	
	public function setInvitationId($v1, $raw=false) {
		$this->setFieldValue('invitation_id', $v1, $raw); 
	}
	
		
	public function getHash() {
		return $this->getFieldValue('hash');
	}
	
	public function setHash($v1, $raw=false) {
		$this->setFieldValue('hash', $v1, $raw); 
	}
	
		
	public function getEmail() {
		return $this->getFieldValue('email');
	}
	
	public function setEmail($v1, $raw=false) {
		$this->setFieldValue('email', $v1, $raw); 
	}
	
		
	public function getName() {
		return $this->getFieldValue('name');
	}
	
	public function setName($v1, $raw=false) {
		$this->setFieldValue('name', $v1, $raw); 
	}
	
		
	public function getUserId() {
		return $this->getFieldValue('user_id');
	}
	
	public function setUserId($v1, $raw=false) {
		$this->setFieldValue('user_id', $v1, $raw); 
	}
	
		
	public function getSiteId() {
		return $this->getFieldValue('site_id');
	}
	
	public function setSiteId($v1, $raw=false) {
		$this->setFieldValue('site_id', $v1, $raw); 
	}
	
		
	public function getBecomeMember() {
		return $this->getFieldValue('become_member');
	}
	
	public function setBecomeMember($v1, $raw=false) {
		$this->setFieldValue('become_member', $v1, $raw); 
	}
	
		
	public function getToContacts() {
		return $this->getFieldValue('to_contacts');
	}
	
	public function setToContacts($v1, $raw=false) {
		$this->setFieldValue('to_contacts', $v1, $raw); 
	}
	
		
	public function getMessage() {
		return $this->getFieldValue('message');
	}
	
	public function setMessage($v1, $raw=false) {
		$this->setFieldValue('message', $v1, $raw); 
	}
	
		
	public function getAttempts() {
		return $this->getFieldValue('attempts');
	}
	
	public function setAttempts($v1, $raw=false) {
		$this->setFieldValue('attempts', $v1, $raw); 
	}
	
		
	public function getAccepted() {
		return $this->getFieldValue('accepted');
	}
	
	public function setAccepted($v1, $raw=false) {
		$this->setFieldValue('accepted', $v1, $raw); 
	}
	
		
	public function getDelivered() {
		return $this->getFieldValue('delivered');
	}
	
	public function setDelivered($v1, $raw=false) {
		$this->setFieldValue('delivered', $v1, $raw); 
	}
	
		
	public function getDate() {
		return $this->getFieldValue('date');
	}
	
	public function setDate($v1, $raw=false) {
		$this->setFieldValue('date', $v1, $raw); 
	}
	
		
	

}
