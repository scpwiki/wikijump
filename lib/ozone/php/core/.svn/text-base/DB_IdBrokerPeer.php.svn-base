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
 * Id broker peer class.
 *
 */
class DB_IdBrokerPeer extends DB_IdBrokerPeerBase {

	/**
	 * Updates internal data regarding indexes for primary keys.
	 */
	public function updateIndexes(){
		$ents = $this->select();	
		foreach ($ents as $ent){
			// get max value of index in the database
			$query = "SELECT MAX(".$ent->getColumnName().") AS m FROM ".$ent->getTableName(); 	
			$db = Database::connection();
			$result = $db->query($query);
			$row = $result->nextRow();
			$maxIdx = $row['m'];
			if($maxIdx == null){ 
				$ent->setNextFreeIndex(0);
			} else {
				$ent->setNextFreeIndex($maxIdx + 1);
			}
		
			$ent->save();
		}
	}

}
