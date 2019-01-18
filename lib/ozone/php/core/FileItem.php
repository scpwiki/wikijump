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
 * Upload file item.
 *
 */
class FileItem {
	private $name;
	private $tmpName;
	private $size;
	private $type;
	private $error;	
	
	public function __construct($ar){
		$this->name = $ar['name'];
		$this->type = $ar['type'];
		$this->tmpName = $ar['tmp_name'];
		$this->error = $ar['error'];
		$this->size = $ar['size'];
	}
	
	public function getName(){
		return $this->name;	
	}
	
	public function getTmpName(){
		return $this->tmpName;	
	}
	
	public function getSize(){
		return $this->size;	
	}
	
	public function getType(){
		return $this->type;	
	}
	
	/**
	 * Returns the error code.
	 */
	public function getError(){
		return $this->error;	
	}
}
