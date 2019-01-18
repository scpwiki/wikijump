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
 * @category Ozone
 * @package Ozone_Web
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */
 
/**
 * The Session Object
 */
class DB_OzoneSession extends DB_OzoneSessionBase {

	private $serializedData = array();
	
	private $newSession = false;
	
	private $sessionChanged = false;
	
	private $cachedUser = null;
	
	/**
	 * Adds a key-value mapping for the serialized data.
	 * @param mixed $key
	 * @param mixed $value
	 */
	public function setSerialized($key, $value){
		$this->serializedData[$key] = $value;	
		$this->sessionChanged = true;
	}
	
	/**
	 * Gets data from the serialized storage.
	 * @param mixed $key 
	 */
	public function getSerialized($key){
		return $this->serializedData[$key];	
	}
	
	/**
	 * Clears serialized data. If $key in null all the session storage data 
	 * is wiped. Otherwise only $key is deleted.
	 * @param mixed $key 
	 */
	public function clearSerialized($key=null){
		if($key===null){
			$this->serializedData = array();	
		}else{
			unset($this->serializedData[$key]);	
		}
		$this->sessionChanged = true;
	}
	
	/**
	 * Updates serializedDatablock that is used directly for database storage. This method
	 * performs serialization from serializedData.
	 */
	private function updateSerializedDatablock(){
		$this->setFieldModified("serialized_datablock");
		if(count($this->serializedData) > 0){
			$this->setSerializedDatablock(serialize($this->serializedData));
		} else {
			$this->setSerializedDatablock(null);
		}
	}
	
	/**
	 * Updates serializedData from serializedDatablock. The method performs
	 * deserialization.
	 */
	private function updateFromSerializedDatablock(){
		if($this->getSerializedDatablock() !== null ){
			$this->serializedData = unserialize($this->getSerializedDatablock());
		} else {
			$this->serializedData = array();
		}
	}
	
	/**
	 * Default constructor. It handles initial population (from the database row) and
	 * data deserialization.
	 */
	public function __construct($row=null){
		parent::__construct($row);
		$this->updateFromSerializedDatablock();
	}
	
	/**
	 * Saves the session object. It also handles required serialization.
	 */
	public function save(){
		$this->updateSerializedDatablock();
		parent::save();	
	}
	
	public function getSerializedDatablock() {
		$dbType =Database::connection()->getType(); 
		if($dbType == 'pgsql'){
			return pg_unescape_bytea($this->getFieldValue('serialized_datablock'));
		} else {
			return $this->getFieldValue('serialized_datablock');
		}
	}
	
	public function getSerializedData() {
		return $this->serializedData;
	}
	
	public function isNewSession(){
		return $this->newSession;	
	}
	
	public function setNewSession($val){
		$this->newSession = $val;	
	}
	
	public function setUserId($userId){
		parent::setUserId($userId);
		$this->cachedUser = null;
		$this->sessionChanged = true;
	}
	
	public function getOzoneUser(){
		if($this->cachedUser != null){
			return $this->cachedUser;	
		}
		$userId = $this->getUserId();
		if($userId == null) {return null;}
		$user = DB_OzoneUserPeer :: instance()->selectByPrimaryKeyCached($userId);
		$this->cachedUser = $user;	
		return $user;
	}
	
	public function getSessionChanged(){
		return $this->sessionChanged;	
	}
	
	public function setSessionChanged($val){
		$this->sessionChanged = $val;
	}
}
