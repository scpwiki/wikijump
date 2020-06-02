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
 * Database object (class) holds the main database connection.
 */ 
class Database {
	public static $connection;	
	public static $idBroker;
	
	/**
	 * Initialize the database connection.
	 */
	public static function init(){
		
		if(GlobalProperties :: $DATABASE_TYPE=="mysql"){
			$db = new MyConnection();
		}
		if(GlobalProperties :: $DATABASE_TYPE=="pgsql"){
			$db = new PgConnection();
		}
		$db->setServer(GlobalProperties :: $DATABASE_SERVER);
		$db->setPort(GlobalProperties :: $DATABASE_PORT);
		$db->setUser(GlobalProperties :: $DATABASE_USER);
		$db->setPassword(GlobalProperties :: $DATABASE_PASSWORD);
		$db->setDatabase(GlobalProperties :: $DATABASE_NAME);
		$db->connect();
		self :: $connection = $db;	
		
		self::$idBroker = new IdBroker();
		
	}
	
	/**
	 * Returns the active database connection.
	 */
	public static function connection(){
		if(self::$connection == null){
			Database::init();	
		}
		return self::$connection;	
	}
	
	/**
	 * Returns an instance of IdBroker object.
	 */
	public static function idBroker(){
		return self::$idBroker;	
	}
}
