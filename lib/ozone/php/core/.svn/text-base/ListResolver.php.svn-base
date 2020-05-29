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
 * @package Ozone_Web
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * List resolver template service.
 *
 */
class ListResolver extends TemplateService{
	protected $storage = array();

	private function loadList($listName){
		$fileName = PathManager::listSpecFile($listName);
			$xml = simplexml_load_file($fileName);
			
			$optionList = $xml->option;
			$out = array();
			foreach($optionList as $option){
				$out["$option".''] = $option->text[0].'';
			}
			$this->storage["$listName"] = $out;
	}

	public function getValuesArray($listName){
		if(!isset($this->storage["$listName"])){
			$this->loadList($listName);
		}
		return $this->storage["$listName"];	
	}
	
	public function resolveKey($listName, $keyName){
		if(!isset($this->storage["$listName"])){
			$this->loadList($listName);
		}
		return $this->storage["$listName"]["$keyName"];	
	}
	
	public function test(){
		echo "ListResolver tested";
	}
	
}
