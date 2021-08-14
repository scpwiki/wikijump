<?php

namespace Ozone\Framework\DB;




use Illuminate\Support\Facades\Auth;
use Ozone\Framework\Database\Database;
use Wikidot\DB\OzoneSessionBase;

use Wikijump\Models\User;

/**
 * The Session Object
 */
class OzoneSession extends OzoneSessionBase {

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
		if(is_countable($this->serializedData) && count($this->serializedData) > 0){
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

	public function setUserId($userId, $raw = false){
		parent::setUserId($userId, $raw);
		$this->cachedUser = null;
		$this->sessionChanged = true;
	}

	public function getOzoneUser(){
        return Auth::user();
	}

	public function getSessionChanged(){
		return $this->sessionChanged;
	}

	public function setSessionChanged($val){
		$this->sessionChanged = $val;
	}
}
