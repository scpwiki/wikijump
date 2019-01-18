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
 * @category Wikidot
 * @package Wikidot
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

abstract class CacheableModule extends SmartyModule {

	protected $timeOut = 0;
	protected $minTimeOut = 0; // important when $allowChangeTimeOut == true
	protected $maxTimeOut = 86400; // --||--
	
	/**
	 * Whether timeout can be changed e.g. by passing timeout="1212" parameter in wiki source.
	 */
	protected $allowChangeTimeOut = false; 
	
	/**
	 * Overrides original method and adds caching mechanisms.
	 */
	public function render($runData){
		
		$pl = $runData->getParameterList();
		$site = $runData->getTemp("site");
		// first determine: to cache or not to cache:
		$uTimeOut = $pl->getParameterValue("timeout");
		if($this->allowChangeTimeOut == true && $uTimeOut != null && $uTimeOut >0){
			$timeOut = $uTimeOut;
			// confront with max and min	
			if($timeOut > $this->maxTimeOut){$timeOut = $this->maxTimeOut;}
			if($timeOut < $this->minTimeOut){$timeOut = $this->minTimeOut;}
		} else {
			$timeOut = $this->timeOut;	
			// do not check max and min - we should trust this value and do not complicate things
		}
		
		if($timeOut != null && $timeOut > 0){
			// cacheable ;-)
			$parmSubKey = md5(serialize($pl->asArray()));
			
			$mcKey = 'module..'.$site->getSiteId().'..'.get_class($this).'..'.$parmSubKey;

			// get the content ;-)
			$mc = Ozone::$memcache;
			$out = $mc->get($mcKey);
			if($out != false){
				return $out;	
			} 
			
			$storeLater = true;	
			
			$out = parent::render($runData);
			
			if($storeLater){
				$mc->set($mcKey, $out, 0, $timeOut);	
			}
			
			return $out;	
		}else{
			return 	parent::render($runData);
		}
	}	
}
