<?php

namespace Ozone\Framework\Database;


use Ozone\Framework\IdBroker;
use Wikidot\Utils\GlobalProperties;

/**
 * Database object (Class) holds the main database connection.
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
