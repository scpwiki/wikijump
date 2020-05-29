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
 * @package Ozone_Form
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */
 
/**
 * A gelper class for XML data storage (definition) of the forms. The only
 * reason for this class is to make the Form object lighter and to store only
 * necessary information and NOT parsed xml with the form definition.
 */
class FormXMLStorage {
	public static $storage = array();
	
	public static function initForm($formName){
		$fileName = PathManager::formSpecFile($formName);
		$xml = simplexml_load_file($fileName);
		
		self::$storage["$formName"]=array();
		$formxml = $xml->form[0];
		self::$storage["$formName"]['xml'] = $formxml;
		
		// refactor just a bit for an easy access to fields
		$fields = array();
		$fieldNames = array();
		foreach ($formxml as $field){
			$tname = $field['name'];
			$fieldNames[] = $tname;
			$fields["$tname"] = $field;
		}
		self::$storage["$formName"]['fields'] = $fields;
		self::$storage["$formName"]['fieldNames'] = $fieldNames;
	}
	
	public static function getFormXML($formName){
		if(self::$storage["$formName"] == null){
			self::initForm($formName);	
		}
		return self::$storage["$formName"]['xml'];	
	} 
	
	public static function getFormFields($formName){
		if(self::$storage["$formName"] == null){
			self::initForm($formName);
		}
		return self::$storage["$formName"]['fields'];	
	} 
	
	public static function getFormFieldNames($formName){
		if(self::$storage["$formName"] == null){
			self::initForm($formName);	
		}
		return self::$storage["$formName"]['fieldNames'];	
	}

}
