<?php





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
