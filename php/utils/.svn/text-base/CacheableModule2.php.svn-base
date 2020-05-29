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

/**
 * A better way to cache
 */
abstract class CacheableModule2 extends SmartyModule {
	
	protected $keyBase;
	protected $timeOut = 3600;
	protected $delay = 0;
	
	protected $keyFull = null;
	protected $keyFullTimestamp = null;
	
	public function render($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		
		$parmArray = $pl->asArray();
		$parmHash = md5(serialize($parmArray).$runData->getModuleTemplate());
		
		if($this->keyFull){
			$key = $this->keyFull;
		}else{
			$key = $this->keyBase.'_v..'.$site->getSiteId().'..'.$parmHash;
		}
		if($this->keyFullTimestamp){
			$tkey = $this->keyFullTimestamp;
		}else{
			$tkey = $this->keyBase.'_lc..'.$site->getSiteId(); // last change timestamp
		}
		
		$mc = OZONE::$memcache;
		$struct = $mc->get($key);
		
		$cacheTimestamp = $struct['timestamp'];
		$changeTimestamp = $mc->get($tkey);
		
		if($struct){
			// check the times
			
			if($changeTimestamp && $changeTimestamp <= $cacheTimestamp + $this->delay){
				
				$out = $struct['content'];
				return $out;	
			}
		}
		
		$out = parent::render($runData);
		
		// and store the data now
		$struct = array();
		$now = time();
		$struct['timestamp'] = $now;
		$struct['content'] = $out;
		
		$mc->set($key, $struct, 0, $this->timeOut);
		
		if(!$changeTimestamp){
			$changeTimestamp = $now;
			$mc->set($tkey, $changeTimestamp, 0, $this->timeOut);
		}

		return $out; 	
	}
	
	protected function _compareMicrotime($t1, $t2){
		$t1 = explode(' ', $t1);
		$t2 = explode(' ', $t2);
		if($t1[1]<$t2[1]){ return -1;}
		if($t1[1]>$t2[1]){ return 1;}
		if($t1[1] == $t2[1]){
			if($t1[0]<$t2[0]){ return -1;}
			if($t1[0]>$t2[0]){ return 1;}
			if($t1[0]==$t2[0]){ return 0;}
		}
	}
	
}
