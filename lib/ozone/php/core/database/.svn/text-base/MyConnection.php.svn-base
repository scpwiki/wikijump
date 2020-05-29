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
 * MySQL database connection resource.
 *
 */
class MyConnection implements DatabaseConnection{
	private $server;
	private $port;
	private $database;

	private $user;
	private $password;

	private $link;
	
	private $type="mysql";

	// construction is based on the GlobalProperties object
	function __construct() {
	}

	function __destruct() {
		if ($link)
			mysql_close($this->link);

	}
	
	public function getType(){
		return $this->type;	
	}
	
	public function setServer($server){
		$this->server=$server;	
	} 
	
	public function setPort($port){
		$this->port = $port;	
	}
	
	public function setUser($user){
		$this->user = $user;	
	}
	
	public function setPassword($password){
		$this->password = $password;	
	}
	
	public function setDatabase($database){
		$this->database = $database;	
	}

	function connect() {
		$this->link = mysql_connect($this->server.":".$this->port, $this->user, $this->password);
		if (!$this->link) {
			throw new OzoneDatabaseException("error: ".mysql_error()."\n");
		}
		$this->selectDatabase($this->database);
		OzoneLogger::instance()->debug("database connection successful");
	}

	function selectDatabase($databaseName) {
		mysql_select_db($databaseName, $this->link);
	}

	function query($query) {
		if(!is_array($query)){
			//if query is empty
			if($query == '' || $query == null) {
				return;
			}
			OzoneLogger::instance()->debug("executing query \"$query\"");
			$result = mysql_query($query);
			if (!$result) {
				throw new Exception("error: ".mysql_error()."\n");
			}
		} else {
			//if query is empty
			if(count($query) == 0){
				return;	
			}
			foreach ($query as $q){
				$result = $this->query($q);	
			}
		}
		return new MyResult($result);

	}

	function tableExists($table) {
		$exists = mysql_query("SELECT 1 FROM `$table` LIMIT 0", $this->link);
		if ($exists)
			return true;
		return false;
	}

	function columnExists($table, $column) {
		$result = $this->query("SHOW COLUMNS FROM ".$table['name'].";");
		while ($row = $result->nextRow()) {
			$columnNames[] = $row['Field'];
		}
		return array_search($column['name'], $columnNames) === true;

	}
}
