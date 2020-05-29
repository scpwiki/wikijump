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
 * Base class mapped to the database table petition_signature.
 */
class PetitionSignatureBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='petition_signature';
		$this->peerName = 'DB_PetitionSignaturePeer';
		$this->primaryKeyName = 'signature_id';
		$this->fieldNames = array( 'signature_id' ,  'campaign_id' ,  'first_name' ,  'last_name' ,  'address1' ,  'address2' ,  'zip' ,  'city' ,  'state' ,  'country' ,  'country_code' ,  'comments' ,  'email' ,  'confirmed' ,  'confirmation_hash' ,  'confirmation_url' ,  'date' );
		
		//$this->fieldDefaultValues=
	}


	
		
	
		
	public function getSignatureId() {
		return $this->getFieldValue('signature_id');
	}
	
	public function setSignatureId($v1, $raw=false) {
		$this->setFieldValue('signature_id', $v1, $raw); 
	}
	
		
	public function getCampaignId() {
		return $this->getFieldValue('campaign_id');
	}
	
	public function setCampaignId($v1, $raw=false) {
		$this->setFieldValue('campaign_id', $v1, $raw); 
	}
	
		
	public function getFirstName() {
		return $this->getFieldValue('first_name');
	}
	
	public function setFirstName($v1, $raw=false) {
		$this->setFieldValue('first_name', $v1, $raw); 
	}
	
		
	public function getLastName() {
		return $this->getFieldValue('last_name');
	}
	
	public function setLastName($v1, $raw=false) {
		$this->setFieldValue('last_name', $v1, $raw); 
	}
	
		
	public function getAddress1() {
		return $this->getFieldValue('address1');
	}
	
	public function setAddress1($v1, $raw=false) {
		$this->setFieldValue('address1', $v1, $raw); 
	}
	
		
	public function getAddress2() {
		return $this->getFieldValue('address2');
	}
	
	public function setAddress2($v1, $raw=false) {
		$this->setFieldValue('address2', $v1, $raw); 
	}
	
		
	public function getZip() {
		return $this->getFieldValue('zip');
	}
	
	public function setZip($v1, $raw=false) {
		$this->setFieldValue('zip', $v1, $raw); 
	}
	
		
	public function getCity() {
		return $this->getFieldValue('city');
	}
	
	public function setCity($v1, $raw=false) {
		$this->setFieldValue('city', $v1, $raw); 
	}
	
		
	public function getState() {
		return $this->getFieldValue('state');
	}
	
	public function setState($v1, $raw=false) {
		$this->setFieldValue('state', $v1, $raw); 
	}
	
		
	public function getCountry() {
		return $this->getFieldValue('country');
	}
	
	public function setCountry($v1, $raw=false) {
		$this->setFieldValue('country', $v1, $raw); 
	}
	
		
	public function getCountryCode() {
		return $this->getFieldValue('country_code');
	}
	
	public function setCountryCode($v1, $raw=false) {
		$this->setFieldValue('country_code', $v1, $raw); 
	}
	
		
	public function getComments() {
		return $this->getFieldValue('comments');
	}
	
	public function setComments($v1, $raw=false) {
		$this->setFieldValue('comments', $v1, $raw); 
	}
	
		
	public function getEmail() {
		return $this->getFieldValue('email');
	}
	
	public function setEmail($v1, $raw=false) {
		$this->setFieldValue('email', $v1, $raw); 
	}
	
		
	public function getConfirmed() {
		return $this->getFieldValue('confirmed');
	}
	
	public function setConfirmed($v1, $raw=false) {
		$this->setFieldValue('confirmed', $v1, $raw); 
	}
	
		
	public function getConfirmationHash() {
		return $this->getFieldValue('confirmation_hash');
	}
	
	public function setConfirmationHash($v1, $raw=false) {
		$this->setFieldValue('confirmation_hash', $v1, $raw); 
	}
	
		
	public function getConfirmationUrl() {
		return $this->getFieldValue('confirmation_url');
	}
	
	public function setConfirmationUrl($v1, $raw=false) {
		$this->setFieldValue('confirmation_url', $v1, $raw); 
	}
	
		
	public function getDate() {
		return $this->getFieldValue('date');
	}
	
	public function setDate($v1, $raw=false) {
		$this->setFieldValue('date', $v1, $raw); 
	}
	
		
	

}
