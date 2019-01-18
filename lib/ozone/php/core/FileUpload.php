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
 * File upload utility.
 *
 */
class FileUpload {

	private $multiple = false;	

	public function __construct(){

	}
	
	public function processUpload(){
		// check if proper upload
		
		//check if multiple
		//is_array($_FILES[])
		// get all uploaded files

	}
	
	public function getFileItem($fieldKey){
		/* a nasty hack follows... because of a strange behaviour of the $_FILES array */
		$ar = array();
		$ar['name'] = $_FILES[$fieldKey]['name'];
		$ar['tmp_name'] = $_FILES["$fieldKey"]['tmp_name'];
		$ar['type'] = $_FILES["$fieldKey"]['type'];
		$ar['size'] = $_FILES["$fieldKey"]['size'];
		$ar['error'] = $_FILES["$fieldKey"]['error'];
		return new FileItem($ar);	
		
	}	
	
}
