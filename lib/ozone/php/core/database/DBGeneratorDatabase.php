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
 * Database generator.
 *
 */
class DBGeneratorDatabase {

	private $tables = array ();
	private $views = array ();
	private $referencer;

	private $sql = array();
	
	private $executeSql = true;

	public function __construct($xml = null) {
		$this->referencer = new DBGeneratorReferencer();
	}

	public function addSchema($xml) {
		foreach ($xml->table as $table) {
			echo "table: ".$table['name']."\n";
			$tname = $table['name'];
			$this->tables["$tname"] = new DBGeneratorTable($table);
			$this->referencer->processXMLTable($table);
		}
		
    	foreach ($xml->view as $view) {
    		echo "view: ".$view['name']."\n";
    		$vname = $view['name'];
    		$this->views["$vname"] = new DBGeneratorView($view);
    	}	
	}

	public function executeSQL() {
		$db = Database::connection();
		global $dropTables;
		foreach ($this->tables as $table) {
			unset($sql);
			if (!$db->tableExists($table->getName())) {
				$sql = $table->generateSQLCreateString();
			} else
				if ($dropTables || $table->getName() == 'ID_BROKER') {
					// note: ID_BROKER should always be dropped
					$sql = "DROP TABLE ".$table->getName() ." CASCADE";
				} else{
					$sql = $table->generateSQLAlterString();
				}
			if($sql){
				$this->sql = array_merge($this->sql, (array)$sql);
				if($this->executeSql){
					$db->query($sql);
				}
			}
		}
		
		foreach ($this->views as $view) {
			unset($sql);
			if (!$db->tableExists($view->getName())) {
				$sql = $view->generateSQLCreateString()."\n";
			} else	{
				$sql = "DROP VIEW ".$view->getName()	;
				$sql = $view->generateSQLCreateString()."\n";
			}
			if($sql){ 
				$this->sql = array_merge($this->sql, (array)$sql);
				if($this->executeSql){
					$db->query($sql);
				}
			}
		}
	}
	
	public function generateClasses(){
		foreach ($this->tables as $table) {
			$table->generateClass();
		}
		foreach ($this->views as $view) {
			$view->generateClass();
		}
	}
	
	public function setupIdBroker(){
		// for each table with a INT-LIKE primary key let the pk be
		// handled by the IdBroker.

		foreach($this->tables as $table){
			$pkColumn = $table->getPkColumn();
			
			if($pkColumn != null && ($pkColumn->isIntLike())){
				echo $pkColumn->getName();
				// check if not already there
				$c = new Criteria();
				$c->add('column_name', $pkColumn->getName());
				$c->add('table_name', $table->getName());
				$r = DB_IdBrokerPeer::instance()->selectOne($c);
				if($r == null){
					$idbe = new DB_IdBroker();
					$idbe->setTableName($table->getName());
					$idbe->setColumnName($pkColumn->getName());	
					$idbe->save();
				}
			}	
			
		}
		
		//in case of regeneration - update the indexes:
		$idbp = DB_IdBrokerPeer::instance();
		$idbp->updateIndexes();
			
	}
	
	/**
	 * Updates references between tables. It is required before the SQL and
	 * class genetation.
	 */
	public function updateReferences(){
//			
//			// add referencing keys to the "primary table"
//				
//
//

		foreach ($this->tables as $table){
			$table->updateReferences($this->referencer);	
		}
	}
	
	public function setExecuteSql($val){
		$this->executeSql = $val;
	}
	
	public function getSql(){
		return $this->sql;
	}
}
