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
 * PostgreSQL databse connection resource.
 *
 */
class PgConnection implements DatabaseConnection{
	private $server;
	private $port;
	private $database;

	private $user;
	private $password;
	
	private $type="pgsql";

	private $link;
	
	private $transactionStarted = false;

	// construction is based on the GlobalProperties object
	function __construct() {

	}

	function __destruct() {
		if ($this->link)
			pg_close($this->link);

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
		$connectionString = "host=".$this->server." port=".$this->port." dbname=".$this->database." user=".$this->user." password=".$this->password;
		ob_start();
		if(GlobalProperties::$DATABASE_USE_PERSISTENT_CONNECTIONS){
			$this->link = pg_pconnect($connectionString);
		} else {
			$this->link = pg_connect($connectionString,PGSQL_CONNECT_FORCE_NEW);
		}
		$error = ob_get_clean();
		if (!$this->link) {
			throw new OzoneDatabaseException($error);
		}
		/* configure the connection */
		@pg_query($this->link, 'SET search_path TO "$user", public, ts2');
		OzoneLogger::instance()->debug("database connection successful");
	}

	function query($query) {
		if(!is_array($query)){
			//if query is empty
			if($query == '' || $query == null) {
				return;
			}
			
			$time_start = microtime(true);
			$result = pg_query($this->link,$query);
			if (!$result) {
				OzoneLogger::instance()->error("execution of query  \"$query\" failed");
				throw new OzoneDatabaseException("error: ".pg_last_error($this->link)."\n");
			}
			$time_end = microtime(true);
			$t = $time_end - $time_start;
			OzoneLogger::instance()->debug("executed query \"$query\" ($t sec)");
		} else {
			//if query is empty
			if(count($query) == 0){
				return null;	
			}
			foreach ($query as $q){
				$result = $this->query($q);	
			}
		}
		return new PgResult($result);

	}

	function tableExists($table) {
		$q = "SELECT table_name FROM information_schema.tables where table_catalog='".db_escape_string(GlobalProperties::$DATABASE_NAME)."' AND table_schema='public' AND table_name='".db_escape_string($table)."'";
		$exists = pg_query($this->link, $q);
		if (pg_fetch_assoc($exists))
			return true;
		return false;
	}

	function columnExists($table, $column) {
		$q = "SELECT column_name FROM information_schema.columns where table_catalog='".db_escape_string(GlobalProperties::$DATABASE_NAME)."' AND table_schema='public' AND table_name='".db_escape_string($table)."' AND column_name='".db_escape_string($column)."'";
		$exists = pg_query($this->link, $q);
		if (pg_fetch_assoc($exists)) {
			return true;
		}
		return false;

	}
	
	public function begin(){
		if(!$this->transactionStarted){
			$this->query("BEGIN WORK");	
			$this->transactionStarted = true;
		}
	}
	
	public function commit(){
		if($this->transactionStarted == true){
			$this->query("COMMIT");
			$this->transactionStarted = false;
		}
	}
	
	public function rollback(){
		if($this->transactionStarted == true){
			$this->query("ROLLBACK");
			$this->transactionStarted = false;
		}
	}
	
	/**
	 * Obtains a lock with a given key. Works only within transaction.
	 */
	public function lock($key){
		$q = "SELECT key FROM ozone_lock WHERE key='".db_escape_string($key)."' FOR UPDATE";
		$r = pg_query($this->link,$q);
		$row = pg_fetch_row($r);
		if($row == false){
			// key does not exist?
			$q2 = "INSERT INTO ozone_lock (key) VALUES ('".db_escape_string($key)."')";
			pg_query($this->link,$q2);
			// try again
			$r = pg_query($q);	
		}
		return true;
	}
	
	public function getLink(){
	    return $this->link;
	}
}
