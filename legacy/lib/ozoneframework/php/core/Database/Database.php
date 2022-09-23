<?php

namespace Ozone\Framework\Database;

use Wikidot\Utils\GlobalProperties;

/**
 * Database object (Class) holds the main database connection.
 */
class Database {
	public static $connection;

	/**
	 * Initialize the database connection.
	 */
	public static function init(){

        $db = new PgConnection();
		$db->setServer(GlobalProperties :: $DATABASE_SERVER);
		$db->setPort(GlobalProperties :: $DATABASE_PORT);
		$db->setUser(GlobalProperties :: $DATABASE_USER);
		$db->setPassword(GlobalProperties :: $DATABASE_PASSWORD);
		$db->setDatabase(GlobalProperties :: $DATABASE_NAME);
		$db->connect();
		self :: $connection = $db;

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

}
