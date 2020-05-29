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
 * Simple date class.
 */
class ODate {
	private $timestamp;
	
	public function __construct($timestamp=null){
		if($timestamp === null){
			$timestamp=time();
		} else {
			if(!is_numeric($timestamp)){
				// not a timestamp
				$timestamp = strtotime($timestamp);	
			}	
		}
		$this->timestamp = $timestamp;	
	}
	
	public function getTimestamp(){
		return $this->timestamp;	
	}
	
	public function setTimestamp($timestamp){
		$this->timestamp = $timestamp;	
	}
	
	public function convertToTZ($diff, $dst=true){
		if($dst){
			$diff += date('I', $this->timestamp);
		}
		$this->timestamp -= $diff*60*60;
	}
	
	public function getDateArray(){
		return getdate($this->timestamp);	
	}
	
	public function getDate($format=null){
		if($format === null){
			$format = 'c';	
		}
		return date($format, $this->timestamp);	
	}
	
	public function addSeconds($sec){
		$this->timestamp+=$sec;	
		return $this;
	}
	
	public function subtractSeconds($sec){
		$this->timestamp-=$sec;	
		return $this;
	}
	
	public function before($date){
		if($this->timestamp < $date->getTimestamp()){
			return true;
		} else {
			return false;
		}	
	}
	
	public function after($date){
		if($this->timestamp > $date->getTimestamp()){
			return true;
		} else {
			return false;
		}	
	}
	
	public function equals($date){
		if($this->timestamp === $date->getTimestamp()){
			return true;
		} else {
			return false;
		}	
	}
	
	public function format($format){
		return strftime($format, $this->timestamp);	
	}

}
