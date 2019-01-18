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
 * Database column generator.
 *
 */
class DBGeneratorColumn {
	
	private $name;
	private $type;
	private $defaultValue = null;
	private $canNull = true;
	private $primaryKey = false;
	private $unique = false;

	public function __construct($column_xml){
		$this->name = $column_xml['name'];
		$this->type = $column_xml['type'];	
		if($column_xml['null'] == 'no' OR $column_xml['null'] == 'false'){
			$this->canNull = false;	
		}
		if($column_xml['primaryKey'] == 'yes' || $column_xml['primaryKey'] == 'true'){
			$this->primaryKey = true;
		}
		if($column_xml['unique'] == 'yes' || $column_xml['unique'] == 'true'){
			$this->unique = true;
		}
		
		if(isset($column_xml['default'])){
			$this->defaultValue = 	$column_xml['default'];
		}
		
	}
	
	public function generateSQLPropertyString(){
		$out = $this->name." ".$this->type." ";
		if($this->canNull === false){
			$out .= " NOT NULL ";	
		}	
		if($this->defaultValue !== null ){
			$out .= "DEFAULT '" . $this->defaultValue ."' ";	
		}
		if($this->primaryKey == true){
			$out .= "PRIMARY KEY";	
		}
		if($this->unique == true){
			$out .= " UNIQUE ";	
		}
		return $out;
	}
	
	public function getName(){
		return $this->name;	
	}
	
	public function getType(){
		return $this->type;	
	}
	
	public function getPropertyName(){
		return underscoreToLowerCase($this->name);
	}
	
	public function getPropertyNameFirstCapitalized(){
		return capitalizeFirstLetter(underscoreToLowerCase($this->name));
	}
	
	public function isPrimaryKey(){
		return $this->primaryKey;	
	}
	public function setPrimaryKey($val){
		$this->primaryKey = $val;	
	}
	public function isUnique(){
		return $this->unique;	
	}
	public function setUnique(){
		return $this->unique;
	}
	
	public function getDefaultValue(){
		return $this->defaultValue;	
	}

	public function isIntLike(){
		$pos1 = strpos($this->type, 'int');
		$pos2 = strpos($this->type, 'INT');
		if($pos1 !== false || $pos2 !== false  ){
			return true;
		} else {
			return false;
		}	
	}
	
}
