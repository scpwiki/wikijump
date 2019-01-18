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
 * Utility class for providing unique strings.
 *
 */
class UniqueStrings {
 	private static $tr_lastTime;
 	private static $tr_lastTimeIssuedNumber = 0;
 	
 	/**
 	 * Returns time-based + int element string.
 	 */
 	public static function timeBased(){
 		$timePart = time().'';
 		// TRANSACTION OR TABLE LOCK SHOULD START HERE
 		$db = Database::connection();
 		$db->begin();
 		$c = new Criteria();
 		$c->setForUpdate(true);
 		$index = DB_UniqueStringBrokerPeer::instance()->selectOne($c);
 		if($index != null){
 			$idx = $index->getLastIndex();
 			$number = $idx + 1;
 			//update index + 1
 			DB_UniqueStringBrokerPeer::instance()->increaseIndex();
 		} else {
 			$number = 0;
 			DB_UniqueStringBrokerPeer::instance()->init();
 		}
 		$db->commit();
 		// TRANSACTION OR TABLE LOCK SHOULD END HERE
 		return $timePart."_".$number;
 	}
 	
 	public static function resetCounter(){
 		DB_UniqueStringBrokerPeer::instance()->reset();	
 	}
 }
