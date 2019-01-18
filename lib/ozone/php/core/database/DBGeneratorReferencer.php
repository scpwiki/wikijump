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
 
class DBGeneratorReferencer {
	private $references = array();

	public function addReference($primaryTableName, $primaryKeyName, $referencingTableName, $referencingKeyName, $customFunction=null){
		if($primaryTableName != $referencingTableName){
			$entry = array();
			$entry['primary_table'] = $primaryTableName;
			$entry['primary_key'] = $primaryKeyName;
			$entry['referencing_table'] = $referencingTableName;
			$entry['referencing_key'] = $referencingKeyName;
			$entry['custom_function'] = $customFunction; 
			$this->references[] = $entry;
		}
	}
	
	public function getReferences(){
		return $this->references;	
	}
	
	public function processXMLTable($xmlTable){
		$freferences = $xmlTable->foreignReference;
		foreach ($freferences as $fr){
			$this->addReference($fr['foreignTable'], $fr['foreignKey'], $xmlTable['name'], $fr['localKey'], $fr['customFunction'] );
			OzoneLogger::instance()->debug("found reference: M: ".$fr['foreignTable'].".". $fr['foreignKey'].", S: ". $xmlTable['name'].".".$fr['localKey']. " ". $fr['customFunction']);
		}
	}
	
}
