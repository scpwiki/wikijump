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

class AnonymousInfoWinModule extends SmartyModule {
	
	public function build($runData){
		$userString = $runData->getParameterList()->getParameterValue("userString");
		
		// check if matches.
		if(preg_match("/^((?:[0-9]{1,3}\.){3}[0-9]{0,3})(?:\|((?:[0-9]{1,3}\.){3}[0-9]{0,3}))?$/", $userString) == 0){
			throw new ProcessException("Bad data");
		}
		
		list($ip, $proxy) = explode("|", $userString);
		
		$runData->contextAdd("ip", $ip);
		$runData->contextAdd("proxy", $proxy);
		
		// check if IP comes from a private range
		// 10.*.*.*, 172.16.*.*,  192.168.*.*, 127.*.*.*, 169.254.*.*

		if(preg_match("/^(10\..*)|(172\.16\..*)|(192\.168\..*)|(127\..*)|(169\.254\..*)/", $ip) !=0){
			$runData->contextAdd("privateIp", true);	
		}		
		
		$runData->contextAdd("userString", $userString);
		
	}
	
}
