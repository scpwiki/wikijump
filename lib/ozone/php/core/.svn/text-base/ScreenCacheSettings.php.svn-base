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
 * Settings for the screen cache.
 *
 */
class ScreenCacheSettings {
	
	protected $anonymousLayoutTimeout;
	protected $loggedLayoutTimeout ;
	protected $anonymousScreenTimeout;
	protected $loggedScreenTimeout;
	
	public function getLayoutTimeout($runData){
		if($runData->isUserAuthenticated()){
			$timeout = $this->loggedLayoutTimeout;
		} else {
			$timeout = $this->anonymousLayoutTimeout;
		}
		return $timeout;
	}
	
	public function isLayoutCacheable($runData){
		$timeout = 	$this->getLayoutTimeout($runData);
		if($timeout == null || $timeout == 0){
			return false;
		} else {
			return true;	
		}
	}
	
	public function getScreenTimeout($runData){
		if($runData->isUserAuthenticated()){
			$timeout = $this->loggedScreenTimeout;
		} else {
			$timeout = $this->anonymousScreenTimeout;
		}
		return $timeout;
	}
	
	public function isScreenCacheable($runData){
		$timeout = 	$this->getScreenTimeout($runData);
		if($timeout == null || $timeout == 0){
			return false;
		} else {
			return true;	
		}
	}

	public function setAnonymousLayoutTimeout($time){
		$this->anonymousLayoutTimeout = $time;	
	}
	
	public function setAnonymousScreenTimeout($time){
		$this->anonymousScreenTimeout = $time;	
	}
	
	public function setLoggedLayoutTimeout($time){
		$this->loggedLayoutTimeout = $time;	
	}
	
	public function setLoggedScreenTimeout($time){
		$this->loggedScreenTimeout = $time;	
	}
	
}
