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

class LoginAction extends SmartyAction {
	
	public function perform($r){}
	
	public function loginEvent($runData){
		$pl = $runData->getParameterList();
		$uname = $pl->getParameterValue("name");
		$upass = $pl->getParameterValue("password");
		
		$userId = $pl->getParameterValue("welcome");
		
		$keepLogged = $pl->getParameterValue("keepLogged");
		$bindIP = $pl->getParameterValue("bindIP");
		
		// decrypt! woooohhooooo!!!!!!!!
		
		$seed = $runData->sessionGet("login_seed");
		
		if($seed == null){
			throw new ProcessException(_("You have been inactive quite long while trying to log in and your session data have expired. Please try to click 'log in' once again."), "no_seed");
		}
		
		$uname = CryptUtils::rsaDecrypt($uname);
		$upass = CryptUtils::rsaDecrypt($upass);
		
		// remove seed
		if(preg_match('/^'.$seed.'/', $uname) == 0 || preg_match('/^'.$seed.'/', $upass) == 0){
			EventLogger::instance()->logFailedLogin($uname);
			throw new ProcessException(_("The user and password do not match."), "login_invalid");
		}
		
		$uname = preg_replace('/^'.$seed.'/', '', $uname);
		$upass = preg_replace('/^'.$seed.'/', '', $upass);
		
		if($userId && is_numeric($userId) && $userId >0){
			$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($userId);
			if($user && $user->getPassword() !== md5($upass)){
				$user = null;
			}
		}else{
		
			$user = SecurityManager::authenticateUser($uname, $upass);
		}
		
		if($user == null){
			EventLogger::instance()->logFailedLogin($uname);
			throw new ProcessException(_("The login and password do not match."), "login_invalid");
		}

		$runData->resetSession();
		$session = $runData->getSession();
		$session->setUserId($user->getUserId());
		// set other parameters
		$session->setStarted(new ODate());
		$session->setLastAccessed(new ODate());
		
		$user->setLastLogin(new ODate());
		$user->save();
		
		if($keepLogged){
			$session->setInfinite(true);	
		}
		if($bindIP){
			$session->setCheckIp(true);
		}
		
		setcookie("welcome", $user->getUserId(), time() + 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);
		
		// log event
		EventLogger::instance()->logLogin();
			
	}
	
	public function loginCancelEvent($runData){
		$runData->sessionDel("login_seed");	
	}
	
	public function logoutEvent($runData){
		$db = Database::connection();
		$db->begin();
			EventLogger::instance()->logLogout();
		if($runData->getUser()){
			$userId = $runData->getUser()->getUserId();
		}
		
		$runData->sessionStop();
		
		// be even wiser! delete all sessions by this user from the current IP string!
		if($userId !== null){
			$c = new Criteria();
			$c->add("user_id", $userId);
			$c->add("ip_address", $runData->createIpString());
			// outdate the cache first
			$ss = DB_OzoneSessionPeer::instance()->select($c);
			$mc = OZONE::$memcache;
			foreach($ss as $s){
				$mc->delete('session..'.$s->getSessionId());	
			}
			DB_OzoneSessionPeer::instance()->delete($c);
		}

		$db->commit();
	}
	
}
