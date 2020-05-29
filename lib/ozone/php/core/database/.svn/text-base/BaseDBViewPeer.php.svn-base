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
 * @package Ozone_Db
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */
 
/**
 * Base view peer class.
 *
 */
abstract class BaseDBViewPeer {
	
	public  $tableName;
	public  $objectName;
	public  $fieldNames;
	public $fieldTypes;
	public $primaryKeyName;
	public $defaultValues;

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
		return $result->asObjects($this->objectName);	
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
		$rarray=  $this->selectByCriteria($criteria);
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
	
	public  function selectCustom($query){
		$my = Database::connection();
		$result = $my->query($query);
		return $result->asObjects($objectName);
	}

	/**
	 * Returns a comma-separated list of fields.
	 */
	private function fieldListString(){
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
			if( $fieldType == 'TIMESTAMP' || $fieldType == 'DATETIME'){
				$out.= " UNIX_TIMESTAMP($fieldName) ";
			}	else {
				$out.= " $tableName.$fieldName ";	
			}
			
		}
		
		return $out;
	}	
	
	public function criteriaToQuery($criteria){
		if($criteria == null){
			$criteria = new Criteria();	
		}
		$db = Database::connection();
		// assemble a query string.
		// if exactQuery is specified - no problem. just run it. responsibility is on
		// the user ;-)
		if($criteria->getExplicitQuery() != null){
			
			return $criteria->getExplicitQuery();	
		}
		// ok - otherwise now we should counstruct the query
		$q = "SELECT ";
		if($criteria->isDistinct()){
			$q.= " DISTINCT ";
		}
		// check if the query has an explicit list of fields to selec
		if($criteria->getExplicitFields() != null){
			$q .= 	$criteria->getExplicitFields();
		} else {
			$q .= $this->fieldListString();
		}	
		
		if($criteria->getExplicitFrom()!=null){
			$q .= " FROM ".	$criteria->getExplicitFrom();
		} else {
			$q .= " FROM ".$this->tableName;
		}
		
		$whereString = $criteria->whereString();
		if($whereString != null){
			$q .= " WHERE ".$criteria->whereString();
		}
		$q .= " ".$criteria->groupByString();
		$q .= " ".$criteria->orderString();
		$q .= " ".$criteria->limitString();
		if($criteria->isForUpdate()){
			$q.= " FOR UPDATE";
		}
		return $q;
	}
	
	public function getPrimaryKeyName(){
		return $this->primaryKeyName;	
	}

}
