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
 * Global variables service for Smarty.
 */
class GlobalVariablesService extends TemplateService {
	
	protected $serviceName = "globals";
	
	private $storage = array();
	private $runData;
	
	public function __construct($runData){
		$this->runData = $runData;
	}
		
	public function set($key, $value){
		$this->storage[$key] = $value;
	}
	
	public function del($key = null){
		if($key !== null){
			unset($this->storage[$key]);	
		} else {
			$this->storage = array();
		}	
	}
	
	public function get($key){
		return $this->storage[$key];	
	}
	
	public function hasKey($key){
		if($this->storage[$key] !== null){
			return true;	
		}	else {
			return false;	
		}
	}
	
}
