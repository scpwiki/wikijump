<?php

namespace DB;

use \Database;





/**
 * Handles unique string services.
 */
class UniqueStringBrokerPeer extends UniqueStringBrokerPeerBase {

	public function increaseIndex(){
		$query = "UPDATE unique_string_broker SET last_index=last_index+1";
		Database::connection()->query($query);
	}

	public function init(){
		$query = "INSERT INTO unique_string_broker (last_index) values (0)";
		Database::connection()->query($query);
	}

	public function reset(){
		$query = "UPDATE unique_string_broker SET last_index=0";
		Database::connection()->query($query);
	}
}
