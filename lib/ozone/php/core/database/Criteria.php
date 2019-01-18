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
 * Database criteria object.
 *
 */
class Criteria{

	private $distinct = false;

	private $explicitQuery;
	private $explicitWhere;
	private $explicitFrom;
	private $explicitFields;
	
	private $limitOffset;
	private $limitCount;

	private $forUpdate = false;

	private $conditions = array();
	private $order = array();
	private $groupBy = array();
	
	/**
	 * If processing joins the variable is an array with entries (arrays) with keys: 
	 * "localKey", "foreignTable", "foreignKey".
	 */
	private $joins = null;
	
	/**
	 * Alias of addAnd().
	 */
	public function add($columnName, $fieldValue, $relation = "=", $escape=true){
		
		$this->	addAnd($columnName, $fieldValue, $relation, $escape);
	}
	
	/** 
	 * Adds a condition to the query via ADD operator.
	 * @param string $columName
	 * @param string $fieldValue
	 * @param string $relation (defaults to "=")
	 */ 
	public function addAnd($columnName, $fieldValue, $relation = "=", $escape=true){
	
		if($fieldValue === null && $relation == "="){
			$relation = "IS";
			$fieldValue = "NULL";
			$escape=false;
		}
		if($fieldValue === null && $relation == "!="){
			
			$relation = "IS NOT";
			$fieldValue = "NULL";
			$escape=false;
		}

		if($fieldValue === true){
			$fieldValue = "TRUE";
			$escape = false;	
		}
		if($fieldValue === false){
			$fieldValue = "FALSE";
			$escape = false;	
		}
		
		if($fieldValue instanceof ODate){
			$fieldValue = $fieldValue->getDate();	
		}
		
		if($this->explicitWhere != null) {
			$this->explicitWhere = "(".$this->explicitWhere.") AND ";
		}
		$this->explicitWhere .= "$columnName $relation ";
		
		if($escape){
			$this->explicitWhere.='\''.db_escape_string($fieldValue).'\'';
		} else {
			$this->explicitWhere.=$fieldValue;
		}

	}
	
	/** 
	 * Adds a condition to the query.
	 * @param string $columName
	 * @param string $fieldValue
	 * @param string $relation (defaults to "=")
	 */ 
	public function addOr($columnName, $fieldValue, $relation = "=", $escape=true){
		if($fieldValue === null && $relation == "="){
			$relation = "IS";
			$fieldValue = "NULL";
			$escape=false;
		}
		if($fieldValue === null && $relation == "!="){
			
			$relation = "IS NOT";
			$fieldValue = "NULL";
			$escape=false;
		}

		if($fieldValue === true){
			$fieldValue = "TRUE";
			$escape = false;	
		}
		if($fieldValue === false){
			$fieldValue = "FALSE";
			$escape = false;	
		}
		
		if($fieldValue instanceof ODate){
			$fieldValue = $fieldValue->getDate();	
		}
		
		if($this->explicitWhere != null) {
			$this->explicitWhere = "(".$this->explicitWhere.") OR ";
		}
		$this->explicitWhere .= "$columnName $relation ";
		
		if($escape){
			$this->explicitWhere.='\''.db_escape_string($fieldValue).'\'';
		} else {
			$this->explicitWhere.=$fieldValue;
		}
		
	}
	
	public function addCriteriaAnd($criteria){
		$query = "";
		
		if($criteria->whereString() != null && $this->explicitWhere != null){
			$this->explicitWhere = "(".$this->explicitWhere.") AND (".$criteria->whereString().")";
		}	
		if($criteria->whereString() != null && $this->explicitWhere == null){
			$this->explicitWhere = $criteria->whereString();
		}	
		
	}
	
	public function addCriteriaOr($criteria){
		if($criteria->whereString() != null){
			$this->explicitWhere = "(".$this->explicitWhere.") OR (".$criteria->whereString().")";
		}	
	}
	
	/**
	 * Sets the "LIMIT" condition for the query.
	 * @param int $count
	 * @param int $offset
	 */
	public function setLimit($count, $offset=null){
		
			$this->limitOffset = $offset;
			$this->limitCount = $count;
		
	}
	
	/**
	 * Clears LIMIT settings.
	 * 
	 */
	public function clearLimit(){
		$this->limitOffset = null;
		$this->limitCount = null;	
	}
	
	/**
	 * Add order element (ascending)
	 * @param string $columnName 
	 */
	public function addOrderAscending($columnName, $extra = ''){
		$this->order["$columnName"] = "ASC" . ($extra?(' '.$extra):'');
	}
	
	/**
	 * Add order element (descending)
	 * @param string $columnName 
	 */
	public function addOrderDescending($columnName, $extra = ''){
		$this->order["$columnName"] = "DESC" . ($extra?(' '.$extra):'');
	}
	
	/**
	 * Add a "GROUP BY..." element.
	 * @param string $field
	 */
	public function addGroupBy($field){
		$this->groupBy[]=$field;
	}

	public function clearOrder(){
		$this->order = array();
	}
	
	/** 
	 * Allows to use join queries.
	 * @param string $localKey - name of the local key
	 * @param string $foreignKey - name of the foreign key in the format "foreign_table.key"
	 */
	public function addJoin($localKey, $foreignKey, $type = null){
		if($this->joins === null){
			$this->joins = array();	
		}
		$tmp = explode('.', $foreignKey);
		$entry = array('localKey' => $localKey, 'foreignTable' =>$tmp[0], 'foreignKey' => $tmp[1], 'type' => $type);
		$this->joins[] = $entry;
	}
	
	public function getJoins(){
		return $this->joins;	
	}
	/**
	 * Returns rendered "WHERE...." string. No ordering/groupin/limit string
	 */
	public function whereString(){
		return $this->explicitWhere;	
	}
	
	/**
	 * Returns rendered "LIMIT..." string. 
	 * @return string
	 */
	public function limitString(){
		if($this->limitCount != null){
			$out = " LIMIT ".$this->limitCount;
			if($this->limitOffset != null){
				$out .= " OFFSET ".$this->limitOffset;	
			}	
			return $out;
		} else {
			return null;
		}
	}	
	
	/**
	 * Returns rendered "ORDER..." string.
	 * @return string 
	 */
	public function orderString(){
		if(count($this->order) >0){
			$out = 'ORDER BY ';
			$first = true;
			foreach($this->order as $key => $value){
				if(!$first){
					$out .= ", ";
				}else{
					$first = false;
				}
				$out .= " $key $value ";	
			}
			return $out;		
		}else{
			return null;
		}
	}
	
	/**
	 * Renders the "GROUP BY..." string.
	 * @return string
	 */
	public function groupByString(){
		if(count($this->groupBy) >0){
			$out = 'GROUP BY ';
			$out .= implode(', ', $this->groupBy);
			return $out;
		}else{
			return null;
		}
	}
	
	/**
	 * Sets the DISTINCT flag for the query. Can be true/false or expression.
	 * @param mixed $distinct
	 */
	public function setDistinct($distinct){
		$this->distinct = $distinct;	
	}
	
	/** 
	 * Returns thr DISTINCT flag.
	 * @return mixed
	 */
	public function isDistinct(){
		return $this->distinct;	
	}
	
	/** 
	 * Sets the FOR UPDATE flag for the query. This is related to locking.
	 * @param boolean $forUpdate
	 */
	public function setForUpdate($forUpdate){
		$this->forUpdate = $forUpdate;	
	}
	
	/**
	 * Returns true if the FOR UPDATE flag is set.
	 * @returns boolean
	 */
	public function isForUpdate(){
		return $this->forUpdate;
	}
	
	/**
	 * Sets the explicit query. Please not that it bypasses all other 
	 * elements specified by add() etc. The exact query will be executed.
	 * Please use with care.
	 * @param string $query complete SQL query to execute.
	 */
	public function setExplicitQuery($query){
		$this->explicitQuery = $query;
	}
	
	/**
	 * Gets the explicit query (if already set).
	 * @return string
	 */
	public function getExplicitQuery(){
		return $this->explicitQuery;	
	}
	
	/**
	 * Sets the explicit "FORM...." part of the query.
	 * @param string $from
	 */
	public function setExplicitFrom($from){
		$this->explicitFrom = $from;
	}
	
	/**
	 * Gets explicit "FROM..." part of the query.
	 * @return string
	 */
	public function getExplicitFrom(){
		return $this->explicitFrom;	
	}
	
	/**
	 * Explicitely sets the "WHERE <conditions>" part of the query. Please do 
	 * not use this too often since it is not compatible with the
	 * structure addAnd(), addOr() and joins.
	 * @param string $where
	 */
	public function setExplicitWhere($where){
		$this->explicitWhere = $where;
	}
	
	public function setExplicitFields($fields){
		$this->explicitFields = $fields;
	}	
	
	public function getExplicitFields(){
		return $this->explicitFields;	
	}
}
