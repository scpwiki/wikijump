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
 * PostgreSQL database query result.
 *
 */ 
class PgResult implements DatabaseResult{

	private $result;

	public function __construct($pg_result) {
		$this->result = $pg_result;
	}

	public function nextRow() {
		return pg_fetch_assoc($this->result);
	}

	public function resetPosition() {

	}

	public function asObjects($className, $prefetched = null) {
		$out = array();
		// check if has a primary key and make it the array key too
		$peerClassName = $className.'Peer';
		$peer = new $peerClassName;
		$pkn = $peer->getPrimaryKeyName();
		while($line = pg_fetch_assoc($this->result)){
			$obj = new $className($line, $prefetched);
			$obj->setNew(false);
			if($pkn == null){
				$out[] = $obj;
			} else {
				$out[$obj->getFieldValue($pkn)] = $obj;
			}
		}
		return $out;
	}
	
	public function getSize(){
			
	}
	
	public function getResult(){
		return $this->result;	
	}
	
	public function fetchAll(){
		return pg_fetch_all($this->result);	
	}

}
