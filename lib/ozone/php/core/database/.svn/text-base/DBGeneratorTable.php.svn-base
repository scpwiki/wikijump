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
 * Database table generator.
 *
 */
class DBGeneratorTable{
	
	private $name;
	private $columns = array();
	private $pkColumnName;
	/** Relations where this table has the primary key and the other one has a referencing key.
	 * The array has a structure:
	 * each entry is an array with keys: 'foreignTableName', 'foreignKeyName',
	 * 'localKeyName'.*/
	private $masterRelations = array();
	/** Relations where this table has the referencing key and the other one has a primary key.
	 * Structure the same as above. */
	private $foreignRelations = array();
	
	public function __construct($table_xml){
		$this->name = $table_xml['name'];
		foreach ($table_xml->column as $column) {
			$cname = $column['name'];
			$this->columns["$cname"] = new DBGeneratorColumn($column);
			if($column['primaryKey'] == "true" || $column['primaryKey'] == "yes"){
				$this->pkColumnName = 	$column['name'];
			}
			
		}
		//allow only one pk
		
	}
	
	public function generateSQLCreateString(){
		$out = "CREATE TABLE ".$this->name." ";	
		$squery1 = '';
		$isFirst = true;
		foreach($this->columns as $column){
			if(!$isFirst){
				$squery1 .=", ";
			} else {
				$isFirst = false;
			}
			
			$squery1 .= $column->generateSQLPropertyString();
		}
		$out .= " (".$squery1 . " ) ;";
		return $out;
	}
	
	/**
	 * create a wrapper for the table to be able to get the primary index of the inserted 
	 * row
	 */
	public function generateCreateRule(){
		if($this->pkColumnName){
			$ctype = $this->columns["{$this->pkColumnName}"]->getType();
			if( "$ctype" ==="serial"){
				$sequenceName = $this->name.'_'.$this->pkColumnName.'_seq';
				$q = "CREATE OR REPLACE RULE get_pkey_on_insert AS ON INSERT TO ".$this->name." DO" .
						" SELECT currval('$sequenceName') AS id";
				return $q;
			}
		}
	}

	public function generateSQLAlterString(){
		$my = Database::connection();

		$result = $my->query("SELECT column_name  FROM information_schema.columns WHERE table_schema='public' AND table_name='".$this->name."'");
		while ($row = $result->nextRow()) {
			$columnNamesCurrent[] = $row['column_name'];
		}
		
		// now iterate over columns in the XML schema
		foreach($this->columns as $column){
			// now if the column does not exist yet - just add it!
			if(! in_array($column->getName(), $columnNamesCurrent)){
				$sql[] = "ALTER TABLE ".$this->name." ADD ".$column->generateSQLPropertyString();
			} 	
		}
		
		$sql[] = $this->generateCreateRule();
		return $sql;
	}
	
	public function getName(){
		return $this->name;
	}
	
	public function generateClass(){
		echo "generating classes for ".$this->name."\n";
		$smarty = new OzoneSmarty();
		$smarty->left_delimiter = '<{';
		$smarty->right_delimiter = '}>';
		$smarty->assign('className', $this->getNameLowercaseFirstCapitalized());
		
		$smarty->assign('tableName', $this->name);
		
		// put columns into context
		$smarty->assign('columns', $this->columns);
		
		//primary key name
		$smarty->assign('primaryKeyName', $this->pkColumnName);
		
		// peer name
		$peerName = "DB_".$this->getNameLowercaseFirstCapitalized()."Peer";
		$smarty->assign('peerName', $peerName);
		
		//default values
		
		$defvals = array();
		foreach($this->columns as $column){
			if($column->getDefaultValue()!= null){
				$key = $column->getName();
				$defvals["$key"] = $column->getDefaultValue();	
			}	
		}
		$smarty->assign('defaultValues', $defvals);

		// references
		$smarty->assign('masterRelations', $this->masterRelations);
		$smarty->assign('foreignRelations', $this->foreignRelations);
		
		$templateFile = OZONE_ROOT ."/files/dbtemplates/DB_ObjectBaseTemplate.tpl";
		$out = $smarty->fetch($templateFile);	
		$cn = 'DB_'.$this->getNameLowercaseFirstCapitalized().'Base';
		file_put_contents(PathManager::dbClass('/base/'.$cn), $out);
		
		//see if file exists!
		$cn = 'DB_'.$this->getNameLowercaseFirstCapitalized();
		if(!file_exists(PathManager::dbClass($cn))){
		
			$templateFile = OZONE_ROOT ."/files/dbtemplates/DB_ObjectTemplate.tpl";
			$out = $smarty->fetch($templateFile);
			file_put_contents(PathManager::dbClass($cn), $out);
		}
		
		$objectName = "DB_".$this->getNameLowercaseFirstCapitalized();
		$smarty->assign('objectName', $objectName);
		
		$templateFilePeer = OZONE_ROOT ."/files/dbtemplates/DB_ObjectPeerBaseTemplate.tpl";
		$out = $smarty->fetch($templateFilePeer);	
		$cn = 'DB_'.$this->getNameLowercaseFirstCapitalized().'PeerBase';
		file_put_contents(PathManager::dbClass('/base/'.$cn), $out);
		
		//see if file exists!
		$cn = 'DB_'.$this->getNameLowercaseFirstCapitalized().'Peer';
		if(!file_exists(PathManager::dbClass($cn))){
			$templateFile = OZONE_ROOT ."/files/dbtemplates/DB_ObjectPeerTemplate.tpl";
			$out = $smarty->fetch($templateFile);
			file_put_contents(PathManager::dbClass($cn), $out);
		}
	}

	public function updateReferences($referencer){
		$references = $referencer->getReferences();

		foreach($references as $r){
			// check if the 'master'
			if($r['primary_table'] == $this->name){
				$newRef = array('foreignTableName' => $r['referencing_table'], 
				'foreignKeyName' => $r['referencing_key'],
				'localKeyName' => $r['primary_key'],
				'foreignTmp' => capitalizeFirstLetter(underscoreToLowerCase($r['referencing_table'])) );
				$this->masterRelations[] = $newRef;
			} 	
			//check if referencing
			if($r['referencing_table'] == $this->name){
				$newRef	= array('foreignTableName' => $r['primary_table'],
				'foreignKeyName' => $r['primary_key'],
				'localKeyName' => $r['referencing_key'],
				'foreignTmp' => capitalizeFirstLetter(underscoreToLowerCase($r['primary_table'])),
				'customFunction' => $r['custom_function'] );
				$this->foreignRelations[] = $newRef;
			}
			
		}	
	}

	public function getNameLowercase(){
		return underscoreToLowerCase($this->name);
	}
	
	public function getNameLowercaseFirstCapitalized(){
		return capitalizeFirstLetter(underscoreToLowerCase($this->name));
	}
	
	public function getPkColumnName(){
		return $this->pkColumnName;	
	}
	
	public function getPkColumn(){
		$pkColumnName =$this->pkColumnName;
		return $this->columns["$pkColumnName"];	
	}

}
