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
 * @package Ozone_Util
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * Handles unique string services.
 */
class DB_UniqueStringBrokerPeer extends DB_UniqueStringBrokerPeerBase {

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
