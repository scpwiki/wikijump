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

class AbuseFlagAction extends SmartyAction {
	
	public function isAllowed($runData){
		$userId = $runData->getUserId();
		if($userId == null || $userId <1){
			throw new WDPermissionException(_("This option is available only to registered (and logged-in) users."));
		}
		return true;
	}
	
	public function perform($r){}
	
	public function flagPageEvent($runData){
		$pl = $runData->getParameterList();
		
		$path = $pl->getParameterValue("path");
		$toFlag = $pl->getParameterValue("flag");
		if($path == null || $path == ''){
			throw new ProcessException(_("Error processing the request."), "no_path");	
		}
		$site = $runData->getTemp("site"); 	
		
		$user = $runData->getUser();
		
		$db = Database::connection();
		$db->begin();
		
		if($toFlag){
			// flag the page
			
			// check if not flagged already
			$c = new Criteria();
			$c->add("user_id", $user->getUserId());
			$c->add("site_id", $site->getSiteId());
			$c->add("path", $path);
			
			$flag = DB_PageAbuseFlagPeer::instance()->selectOne($c);
			
			if($flag == null){
				$flag = new DB_PageAbuseFlag();
				$flag->setUserId($user->getUserId());
				$flag->setSiteId($site->getSiteId());
				$flag->setPath($path);
				$flag->save();
				EventLogger::instance()->logFlagPage($path);
			}
		}else{
			// unflag	
			$c = new Criteria();
			$c->add("user_id", $user->getUserId());
			$c->add("site_id", $site->getSiteId());
			$c->add("path", $path);
			DB_PageAbuseFlagPeer::instance()->delete($c);
			EventLogger::instance()->logUnflagPage($path);
		}
		
		$db->commit();
		
	}
	
	public function flagUserEvent($runData){
		$pl = $runData->getParameterList();

		$toFlag = $pl->getParameterValue("flag");
		
		$targetUserId = $pl->getParameterValue("targetUserId");
		
		if($targetUserId == null || $targetUserId == '' || !is_numeric($targetUserId)){
			throw new ProcessException(_("Error processing the request."), "no_target_user");	
		}
		
		$targetUser = DB_OzoneUserPeer::instance()->selectByPrimaryKey($targetUserId);
		if($targetUser == null){
			throw new ProcessException(_("Error processing the request."), "no_target_user");	
		}
		
		$site = $runData->getTemp("site"); 	
		
		$user = $runData->getUser();
		
		$db = Database::connection();
		$db->begin();
		
		if($toFlag){
			// flag the user
			
			// check if not flagged already
			$c = new Criteria();
			$c->add("user_id", $user->getUserId());
			$c->add("target_user_id", $targetUser->getUserId());
			
			$flag = DB_UserAbuseFlagPeer::instance()->selectOne($c);
			
			if($flag == null){
				
				$siteId = $site->getSiteId();
				// get the host if any
				$host = $pl->getParameterValue("host");
				if($host){
					
					if(preg_match("/^([a-zA-Z0-9\-]+)\." . GlobalProperties::$URL_DOMAIN_PREG . "$/", $host, $matches)==1){
						$siteUnixName=$matches[1];
						$c = new Criteria();
						$c->add("unix_name", $siteUnixName);
						$siter = DB_SitePeer::instance()->selectOne($c);
					} else {
						$c = new Criteria();
						$c->add("custom_domain", $host);
						$siter = DB_SitePeer::instance()->selectOne($c);	
					}
					
					if($siter !== null){
						$siteId = $siter->getSiteId();	
					}
				}
				
				$flag = new DB_UserAbuseFlag();
				$flag->setUserId($user->getUserId());
				$flag->setSiteId($siteId);
				$flag->setTargetUserId($targetUser->getUserId());
				$flag->save();
				EventLogger::instance()->logFlagUser($targetUser);
			}
		}else{
			// unflag	
			$c = new Criteria();
			$c->add("user_id", $user->getUserId());
			$c->add("target_user_id", $targetUser->getUserId());
			DB_UserAbuseFlagPeer::instance()->delete($c);
			EventLogger::instance()->logUnflagUser($targetUser);
		}
		
		$db->commit();
		
	}
	
	public function flagAnonymousEvent($runData){
		$pl = $runData->getParameterList();
		
		$toFlag = $pl->getParameterValue("flag");
		
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
		
		$db = Database::connection();
		$db->begin();
		
		$ips = explode('|',$userString);

		if($toFlag){
			$i = 0;
			foreach($ips as $ip){
				
				$i++;
				if(false && preg_match("/^(10\..*)|(172\.16\..*)|(192\.168\..*)|(127\..*)|(169\.254\..*)/", $ip) !=0){
					continue;	
				}
				// flag the IP
			
				// check if not flagged already
				$c = new Criteria();
				$c->add("user_id", $user->getUserId());
				$c->add("address", $ip);
			
				$flag = DB_AnonymousAbuseFlagPeer::instance()->selectOne($c);
			
				if($flag == null){
				
					$siteId = $site->getSiteId();
					
					$flag = new DB_AnonymousAbuseFlag();
					$flag->setUserId($user->getUserId());
					$flag->setSiteId($siteId);
					$flag->setAddress($ip);
					if($i == 2){
						$flag->setProxy(true);	
					}
					$flag->save();
				}
			}
			
			EventLogger::instance()->logFlagAnonymous($userString);
			
		}else{
			foreach($ips as $ip){
				// 	unflag	
				$c = new Criteria();
				$c->add("user_id", $user->getUserId());
				$c->add("address", $ip);
				DB_AnonymousAbuseFlagPeer::instance()->delete($c);
			}
			EventLogger::instance()->logUnflagAnonymous($userString);
		}
		
		$db->commit();
		
	}
	
}
