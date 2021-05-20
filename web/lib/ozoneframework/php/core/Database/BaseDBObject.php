<?php

namespace Ozone\Framework\Database;



use Ozone\Framework\ODate;

/**
 * Base object for all database OM objects representing tables. It implements
 * most of the database logic - getting/setting field valuse, inserting/updating etc.
 */
abstract class BaseDBObject {

	/**
	 * List of the fields of this particular object/table.
	 * @var array
	 */
	protected $fieldNames;

	/**
	 * Contains the values of the fields.
	 * @var array
	 */
	protected $fieldValues = array();

	/**
	 * Lists modified (by setters) fields.
	 * @var array
	 */
	protected $modifiedFields = array();

	/**
	 * Default values as defined in the -db.xml files.
	 * @var array
	 */
	protected $fieldDefaultValues = array();

	/**
	 * Name of the corresponding table.
	 * $var string
	 */
	protected $tableName;

	/**
	 * Is the object new? It affects the way save() method works. If object "is new" then
	 * save() works as INSERT. Otherwise it acts as UPDATE.
	 * @var boolean
	 */
	protected $isnew = true;

	/**
	 * Name of the primary key (column) in the table. At this point primary key can
	 * be constituted only from single column.
	 * @var string
	 */
	protected $primaryKeyName;

	/**
	 * Class name of the corresponding "Peer".
	 * @var string
	 */
	protected $peerName;

	/**
 	* Temporary storage - used to store data associated with object but not written into
 	* the database. That is why it is calles "temporary".
 	* @var array
 	*/
	protected $temporaryStorage = array();

	/**
	 * If using joins the sourceRow holds original data used to populate the object.
	 * It is than used to populate any object that was fetched from the join query.
	 */
	protected $sourceRow = null;

	/**
	 * If using joins this holds an array with table names that were fetched with the select query.
	 */
	protected $prefetched = null;

	/**
	 *  When joining tables this array holds objects already fetched by getSomeTable() method
	 * such that subsequent get...() requests do not query database again.
	 */
	protected $prefetchedObjects = null;

	/**
	 * Array of field names which have been set and should be saved as "raw".
	 */
	protected $raw = array();

	/**
	 * Function used to set values of $tableName, $fieldDefaultValues
	 * and $fieldNames.
	 */
	protected abstract function internalInit();

	/**
	 * Default constructor. If $row is non-null then object properties are filled with
	 * values of the row. If $row is null, object properties are filled with default values.
	 * @param array $row initial values for the object
	 */
	public function __construct($row = null, $prefetched = null) {
		$this->internalInit();
		if ($row != null) {
			$this->populate($row);
			if($prefetched != null){
				$this->sourceRow = $row;
				if($prefetched != null){
					$this->prefetched = $prefetched;
					$this->prefetchedObjects = array();
				}
			}
		} else {
			// default values shoud be set HERE!!!
			$peerName = $this->peerName;
			$peer = new $peerName();
			$defvals = $peer->getDefaultValues();
			foreach($defvals as $key => $value){
				// handle special types

				if($value === "true"){
					$value = true;
				}
				if($value === "false"){
					$value = false;
				}
				$this->fieldValues["$key"]=$value;
			}

		}

	}

	/**
	 * Populate object data with values from the $row.
	 * @param array $row values to fill the object data
	 */
	public function populate($row) {
		$peerName = str_replace("_", "\\", $this->peerName);
		$peer = new $peerName();
		foreach ($this->fieldNames as $field) {
			$fieldType = $peer->getFieldType($field);

			$val = $row[$this->tableName."___".$field] ?? $row[$field];

			// handle date object (timestamp)

			if(strtoupper($fieldType) == "TIMESTAMP"){
				$val = new ODate($val);
				// for sure again set TZ to UTC
			}

			if($val == "NULL"){
				$val = null;
			}

			// handle boolean
			if(strtoupper($fieldType) == "BOOLEAN"){
				if($val == "T" || $val == "t" || $val == "TRUE"){
					$val = true;
				} elseif($val == "F" || $val == "f" || $val == "FALSE"){
					$val = false;
				} else {
					$val = null;
				}
			}

			$this->fieldValues[$field] = $val;
		}
	}

	/**
	 * Obtains primary key value for the object. This can be used in case you do not want
	 * to store object in the database yet but want to assign primary key value immediately.
	 */
	public function obtainPK() {

		$pkName = $this->primaryKeyName;
		$sequenceName = $this->tableName.'_'.$pkName.'_seq';
		$db = Database::connection();
		$q = "SELECT nextval('$sequenceName') AS nextval";
		$row = $db->query($q)->nextRow();
		$idx = $row['nextval'];

		$this->fieldValues["$pkName"] = $idx;
	}

	/**
	 * Sets the field as (un)modified. Only fields marked as modified are updated when
	 * performing save/update. Setters set fields as modified by default.
	 * @param string $fieldName name of the field/property (capitalized)
	 * @param boolean $modified
	 */
	public function setFieldModified($fieldName, $modified=true){
		if($modified==true){
		    if(array_key_exists("$fieldName", $this->modifiedFields)) {
                // if not already modified, set 'modified'
                if (!$this->modifiedFields["$fieldName"] == true) {
                    $this->modifiedFields["$fieldName"] = true;
                }
            }
		}else{
			unset($this->modifiedFields["$fieldName"]);
		}
	}

	/**
	 * Checks if the field is mofified.
	 * @param string $fieldName
	 * @return boolean true if modified
	 */
	public function getFieldModified($fieldName){
		$val = $this->modifiedFields["$fieldName"];
		if($val){
			return true;
		}else {
			return false;
		}
	}

	/**
	 * Returns all modified fields
	 * @return array array of modified fields
	 */
	public function getModifiedFields(){
		return array_keys($this->modifiedFields);
	}

	/**
	 * Sets value of the field.
	 * @param string $fieldName name of the field
	 * @param mixed $fieldValue value for the field
	 */
	public function setFieldValue($fieldName, $fieldValue, $raw = false){
		if($raw){
			$this->raw[$fieldName] = true;
		}else{
			unset($this->raw[$fieldName]);
		}
		$this->setFieldModified($fieldName);
		$this->fieldValues[$fieldName] = $fieldValue;
	}

	/**
	 * Gets field value.
	 * @param string $fieldName name of the field.
	 * @return mixed
	 */
	public function getFieldValue($filedName){
		return $this->fieldValues[$filedName] ?? null;
	}

	/**
	 * Gets all field values as an array.
	 * @return array array of the field values.
	 */
	public function getFieldValuesArray(){
		return $this->fieldValues;
	}

	/**
	 * Gets the 'is new' status.
	 * @return boolean
	 */
	public function isNew() {
		return $this-> isnew;
	}

	/**
	 * Sets the 'is new' status.
	 * @param boolean $isnew
	 */
	public function setNew($isnew) {
		$this-> isnew = $isnew;
	}

	/**
	 * Checks if a field should be saved in a "raw" form.
	 */
	public function isFieldRaw($fieldName){
		if(isset($this->raw[$fieldName])){
			return true;
		}else{
			return false;
		}
	}

	/**
	 * Saves the object into database. Based on the 'is new' status an UPDATE or INSERT
	 * is performed. Internally the save() method on the Peer object is called.
	 */
	public function save() {

		$pn = $this->peerName;
		$a = new $pn;
		$a->save($this);
		$this->isnew = false;
	}

	/**
	 * Get value from the temporary storage.
	 * @param mixed $key
	 * @return mixed
	 */
	public function getTemp($key){
		return array_key_exists($key, $this->temporaryStorage) ? $this->temporaryStorage[$key] : null;
	}

	/**
	 * Sets key-value pair in the temporary storage
	 * @param mixed $key
	 * @param mixed $value
	 */
	public function setTemp($key, $value){
		$this->temporaryStorage[$key] = $value;
	}

	/**
	 * Clears temporary storage. If $key is supplied - only one key-value is cleared.
	 * If $key is null - all the storage is cleared.
	 */
	public function clearTemp($key=null){
		if($key == null){
			$this->temporaryStorage = array();
		} else{
			unset($this->temporaryStorage[$key]);
		}
	}

	public function getSourceRow(){
		return $this->sourceRow;
	}

}
