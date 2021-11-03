<?php

namespace Ozone\Framework\Database;


use mysql_xdevapi\DatabaseObject;
use Ozone\Framework\ODate;
use Wikijump\Models\User;

/**
 * Base peer Class.
 *
 */
abstract class BaseDBPeer {

	public  $tableName;
	public  $objectName;
	public  $fieldNames;
	public $fieldTypes;
	public $primaryKeyName;
	public $defaultValues;

	public static function peerForTable(string $tableName)
    {
        //Their jank, not mine.
		$className = 'Wikidot\\DB\\'.capitalizeFirstLetter(underscoreToLowerCase($tableName)).'Peer';

        /**
         * I'm adding some hacky shit here until we can make this function go away.
         * As we're building new tables in Laravel their formula doesn't really work.
         * This translates the new tables to the existing Ozone classes.
         *
         * All these models need the LegacyCompatibility trait attached.
         */
        switch ($tableName) {
            case 'page_contents':
                $className = PageContents::class;
                break;
            case 'users':
                $className = User::class;
                break;
        }

		return new $className;
	}

	/**
	 * Performs internal initialization. This method has to be overriden by
	 * inheriting classes.
	 */
	protected abstract function internalInit();

	/**
	 * Default constructor.
	 */
	public function __construct() {
		$this->internalInit();
	}

	public function selectOneByCriteria($criteria=null){
		$c = clone($criteria);
		$c->setLimit(1);
		$rarray = $this->selectByCriteria($c);
		if(count($rarray) == 1){
			return current($rarray);
		} else {
			return null;
		}
	}

	public function selectByCriteria($criteria=null){
		$db = Database::connection();
		$q = $this->criteriaToQuery($criteria);

		$result = $db->query($q);

		// check if joins
		if($criteria != null) {$joins = $criteria->getJoins();}
		if($joins != null){
			// construct prefetch table with table names
			$prefetched = array();
			foreach($joins as $j){
				$prefetched[] = $j['foreignTable'];
			}
		}
		return $result->asObjects($this->objectName, $prefetched);
	}

	/**
	 * Selects data from the database.
	 */
	public  function selectByExplicitQuery($criteriaQuery=''){
		$q = "SELECT ". $this->fieldListString()." FROM ". $this->tableName;
		$q = $q ." ".$criteriaQuery;
		$db = Database::connection();
		$result = $db->query($q);
		return $result->asObjects($this->objectName);
	}

	/**
	 * Selects only one single row/object from the database.
	 * The result is returned only if there is ONE single row
	 * matching the condition. Otherwise null is returned. Only one object
	 * is returned instead of an array.
	 * @param string criteriaQuery
	 * @return DatabaseObject
	 */
	public  function selectOneByExplicitQuery($criteriaQuery=''){
		$rarray = $this->selectByExplicitQuery($criteriaQuery . " LIMIT 1");
		if(count($rarray) == 1){
			return current($rarray);
		} else {
			return null;
		}
	}

	/**
	 * Selects data from the database.
	 */
	public  function select($criteria=''){
		return $this->selectByCriteria($criteria);
	}

	/**
	 * Selects only one single row/object from the database. For now it simply appends
	 * "LIMIT 1" to the query. If the criteria is not met by any row - null is returned.
	 * @param string criteriaQuery
	 * @return DatabaseObject
	 */
	public  function selectOne($criteria=null){
		if($criteria == null){
			$criteria0 = new Criteria();
		} else {
			$criteria0 = clone($criteria);
		}
		$criteria0->setLimit(1);
		$rarray=  $this->selectByCriteria($criteria0);
		if(count($rarray) == 1){
			return current($rarray);
		} else {
			return null;
		}
	}

	public function selectCount($criteria = null){
		if($criteria == null){
			$criteria = new Criteria();
		} else {
			$criteria = clone($criteria);
		}
		$db = Database::connection();

		$criteria->setExplicitFields("count(*) AS count");
		$q = $this->criteriaToQuery($criteria);
		$result = $db->query($q);
		$row = $result->nextRow();
		return $row['count'];
	}

	public  function selectCountExplicitQuery($criteriaQuery='', $suffixQuery = ''){
		$q = "SELECT count(*) FROM ". $this->tableName . $criteriaQuery ." ".$suffixQuery;
		$my = Database::connection();
		$result = $my->query($q);
		$row = $result->nextRow();
		return $row['count(*)'];
	}

	public function save($object){
		if($object->isNew()){
			$this->insert($object);
		} else {
			$this->update($object);
		}

	}

	protected function insert($object){
		$ovals = $object->getFieldValuesArray();
		$pkName = $this->primaryKeyName;

		if($pkName != null){
			if($ovals[$pkName] == null && $this->fieldTypes[$pkName] === "serial"){
				$object->obtainPK();
				$ovals = $object->getFieldValuesArray();
			}
		}

		$query = "INSERT INTO ".$this->tableName." ";
		$co = 0;

		$colstring = '';
		$valstring = '';

		$db = Database::connection();

		foreach($this->fieldNames as $field){
			if(isset($ovals[$field])){

				if($co >0 ) {
					$colstring .= ", ";
					$valstring .= ", ";
				}

				// check if not instance of ODate
				$val = $ovals[$field];
				if($val instanceof ODate){

					// for sure convert to UTC
					$val=$val->getDate();
				}

				// process 'bytea' type
				$fieldType = $this->fieldTypes["$field"];

				// process boolean type
				if($fieldType == 'boolean'){
					if($val === true){
						$val="TRUE";
					} elseif($val === false) {
						$val = "FALSE";
					} else {
						$val = null;
					}
				}

				// Process the JSONB data type.
				if($fieldType === 'json' && $fieldType === 'jsonb'){
					$val = json_encode($val);
				}

				if($val === null){
					$val = "NULL";
				} else {
					if($fieldType == 'bytea' && $db->getType()=='pgsql'){

						$val = pg_escape_bytea($ovals[$field]);
					}else{
						$val = db_escape_string($val);
					}
					$val = "'$val'";
				}

				// handle raw?
				if($object->isFieldRaw($field)){
					$val = $ovals[$field];
				}

				$colstring.= "$field";
				$valstring.= "$val";

				$co++;

			}
		}
		$query .= "($colstring) VALUES ($valstring)";
		$db->query($query);
	}

	public function update($object){
		$query = "UPDATE ".$this->tableName." ";
		$co = 0;
		$ovals = $object->getFieldValuesArray();
		$fieldsToUpdate = $object->getModifiedFields();
		if(count($fieldsToUpdate) == 0){
			return;
		}
		$db = Database::connection();
		foreach($fieldsToUpdate as $field){

				if($co >0 ) {$query = $query.", ";}else{
					$query = $query. ' SET ';
				}
				$val =  $ovals[$field];

				// check if not instance of ODate
				$val = $ovals[$field];
				if($val instanceof ODate){

					// for sure convert to UTC
					$val=$val->getDate();
				}

				$fieldType = $this->fieldTypes["$field"];

				// process boolean type
				if($fieldType == 'boolean'){

					if($val == true){
						$val="TRUE";
					} elseif($val === false) {
						$val = "FALSE";
					} else {
						$val = null;
					}
				}

				// Process the JSONB data type.
				if($fieldType === 'json' && $fieldType === 'jsonb'){
					$val = json_encode($val);
				}

				if($val === null){
					$val = "NULL";
				} else {
					if($fieldType == 'bytea' && $db->getType()=='pgsql'){

						$val = pg_escape_bytea($ovals[$field]);
					}else{
						$val = db_escape_string($val);
					}
					$val = "'$val'";
				}

				// handle raw?
				if($object->isFieldRaw($field)){
					$val = $ovals[$field];
				}

				$query = $query .$field.'='.$val.' ';
				$co++;

		}
		$query = $query. "WHERE ".$this->primaryKeyName."='". db_escape_string($object->getFieldValue($this->primaryKeyName)) ."' ";

		$db->query($query);
	}

	public function delete($criteria){
		$db = Database::connection();
		$q = $this->criteriaToQueryForDelete($criteria);
		$db->query($q);

	}

	public function deleteOne($criteria){
		if($criteria == null){
			$criteria = new Criteria();
		}
		$criteria->setLimit(1);
		$this->delete($criteria);
	}

	public function getFieldType($fieldName){
		return $this->fieldTypes["$fieldName"];
	}

	/**
	 * Returns a comma-separated list of fields.
	 */
	public function fieldListString(){
		$first = true;
		$out = ' ';

		$tableName = $this->tableName;
		foreach($this->fieldNames as $fieldName){
			if(!$first){
				$out.=' , ';
			} else {
				$first = false;
			}
			$fieldType = $this->fieldTypes["$fieldName"];
			$out.= " $tableName.$fieldName ";
		}

		return $out;
	}

	/**
	 * Returns a comma-separated list of fields.
	 */
	public function fieldListStringSpecialJoin(){
		$first = true;
		$out = ' ';
		$tableName = $this->tableName;
		foreach($this->fieldNames as $fieldName){
			if(!$first){
				$out.=' , ';
			} else {
				$first = false;
			}
			$fieldType = $this->fieldTypes["$fieldName"];
			$out.= " $tableName.$fieldName AS ${tableName}___$fieldName";
		}

		return $out;
	}

	public function getDefaultValues(){
		return $this->defaultValues;
	}

	public function criteriaToQuery($criteria){
		if($criteria == null){
			$criteria = new Criteria();
		}
		$db = Database::connection();
		// assemble a query string.
		// if exactQuery is specified - no problem. just run it. responsibility is on
		// the user
		if($criteria->getExplicitQuery() != null){
			return $criteria->getExplicitQuery();
		}
		// ok - otherwise now we should counstruct the query
		$joins = $criteria->getJoins();
		$q = "SELECT ";
		if($criteria->isDistinct()){
			$q.= " DISTINCT ";
			if($criteria->isDistinct() !== true){
				$q.= " ON (".$criteria->isDistinct().") ";
			}
		}
		// check if the query has an explicit list of fields to selec
		if($criteria->getExplicitFields() != null){
			$q .= 	$criteria->getExplicitFields();
		} else {
			if($joins == null){
				$q .= ' '.$this->tableName.'.* '; //$this->fieldListString();

			}else{
				$q .= ' '.$this->fieldListStringSpecialJoin();
				foreach($joins as $join){
					// get aliased field list
					$peer = self::peerForTable($join['foreignTable']);
					$q .= ', '.$peer->fieldListStringSpecialJoin();
				}
			}
		}

		if($criteria->getExplicitFrom()!=null){
			$q .= " FROM ".	$criteria->getExplicitFrom();
		} else {
			$q .= " FROM ".$this->tableName;
			if($joins != null){
				foreach($joins as $join){
					$q .= ', '.$join['foreignTable'];
				}
			}
		}

		$whereString = $criteria->whereString();

		if($joins != null){
			if($whereString != null){
				$q .= " WHERE (".$whereString.') AND';

			} else {
				$q.=' WHERE ';
			}
			$first = true;
			foreach($joins as $join){
				if($first){$first = false;}else{$q.=" AND ";}
				$q .= ' ';

				if(strpos($join['localKey'], '.') === false)  {
					$q .= $this->tableName.'.';
				}
				$q .= $join['localKey'].' = '.$join['foreignTable'].'.'.$join['foreignKey'].' ';
			}
		}elseif($whereString != null){
			$q .= " WHERE ".$whereString;
		}

		$q .= " ".$criteria->groupByString();
		$q .= " ".$criteria->orderString();
		$q .= " ".$criteria->limitString();
		if($criteria->isForUpdate()){
			$q.= " FOR UPDATE";
		}
		return $q;
	}

	public function criteriaToQueryForDelete($criteria){
		if($criteria == null){
			$criteria = new Criteria();
		}
		$db = Database::connection();
		// assemble a query string.
		// if exactQuery is specified - no problem. just run it. responsibility is on
		// the user
		if($criteria->getExplicitQuery() != null){
			return $criteria->getExplicitQuery();
		}
		// ok - otherwise now we should counstruct the query
		$q = "DELETE ";

			$q .= " FROM ".$this->tableName;

		$whereString = $criteria->whereString();
		if($whereString != null){
			$q .= " WHERE ".$criteria->whereString();
		}

		$q .= " ".$criteria->limitString();

		return $q;
	}

	public function selectByPrimaryKey($primaryKey){
		$q = "WHERE ".$this->primaryKeyName." = '".db_escape_string($primaryKey)."'";
		return $this->selectOneByExplicitQuery($q);

	}

	public function getPrimaryKeyName(){
		return $this->primaryKeyName;
	}

	public function deleteByPrimaryKey($value){
		$c = new Criteria();
		$c->add($this->primaryKeyName, $value);
		$this->delete($c);
	}

}
