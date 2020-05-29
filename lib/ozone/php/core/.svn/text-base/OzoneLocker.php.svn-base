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
 * Db locker class.
 *
 */
class OzonePGLocker {
	// use separate database connection
	public static $connection;
	
	public static function lock($key){
		if(self::$connection == null){
			$connectionString = "host=".$this->server." port=".$this->port." dbname=".$this->database." user=".$this->user." password=".$this->password;
			self::$connection = pg_connect($connectionString,PGSQL_CONNECT_FORCE_NEW);
		}
		pg_query("BEGIN");
		$q = "SELECT key FROM ozone_lock WHERE key='".db_escape_string($key)."' FOR UPDATE";
		$r = pg_query($q);
		$row = pg_fetch_row($r);
		if($row == false){
			// key does not exist?
			$q2 = "INSERT INTO ozone_lock (key) VALUES ('".db_escape_string($key)."')";
			pg_query($q2);
			// try again. I hope no 2 concurent inserts will be done at the some time... :-(
			$r = pg_query($q);	
		}
		return true;
			
	}
	
	public static function release($key){
			
	}
	
}
