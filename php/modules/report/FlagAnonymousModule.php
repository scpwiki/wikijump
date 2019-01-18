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

class FlagAnonymousModule extends SmartyModule {
	
	public function isAllowed($runData){
		$userId = $runData->getUserId();
		if($userId == null || $userId <1){
			throw new WDPermissionException(_("This option is available only to registered (and logged-in) users."));
		}
		return true;
	}
	
	public function build($runData){
		$pl = $runData->getParameterList();
			
		$userString = $pl->getParameterValue("userString");
		if($userString == null || $userString == '' ){
			throw new ProcessException(_("Error processing the request."), "no_user_string");	
		}
		
		// check if userString match the IP pattern
		
		if(preg_match('/^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+(\|[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+)?$/', $userString) !==1){
			throw new ProcessException(_("Error processing the request."), "bad_user_string");	
		}

		$site = $runData->getTemp("site"); 	
		$user = $runData->getUser();

		// which to use which not.
		
		$ips = explode('|',$userString);
		
		$flagged = true;
		
		$valid1 = false;
		
		foreach($ips as $ip){
			// check if private
			if(false && preg_match("/^(10\..*)|(172\.16\..*)|(192\.168\..*)|(127\..*)|(169\.254\..*)/", $ip) !=0){
				continue;	
			}
			$valid1 = true;
			
			$c = new Criteria();
			$c->add("address", $ip);
			$c->add("user_id", $user->getUserId());
			
			$flag = DB_AnonymousAbuseFlagPeer::instance()->selectOne($c);
			if($flag){
				$flagged = $flagged && true;	
			}else{
				$flagged = false;
			}
			
		}
		
		if(!$valid1){
			throw new ProcessException(_("IP address of the user belongs to a private subnet. Sorry, such an address can not be flagged."));	
		}

		if($flagged){
			$runData->contextAdd("flagged", true);	
		}
		
		$runData->contextAdd("userString", $userString);
		list($ip, $proxy) = explode("|", $userString);
		$runData->contextAdd("ip", $ip);
		$runData->contextAdd("proxy", $proxy);

	}
	
}
