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

class Login2Action extends SmartyAction {
	
	public function perform($r){}
	
	public function loginEvent($runData){
		$pl = $runData->getParameterList();
		$uname = $pl->getParameterValue("name");
		$upass = $pl->getParameterValue("password");
		
		$userId = $pl->getParameterValue("welcome");
		
		$keepLogged = $pl->getParameterValue("keepLogged");
		$bindIP = $pl->getParameterValue("bindIP");
		
		// decrypt! woooohhooooo!!!!!!!!
		
		
		if($userId && is_numeric($userId) && $userId >0){
			$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($userId);
			if($user && $user->getPassword() !== md5($upass)){
				$user = null;
			}
		}else{
		
            // allow logging with nick name too
            if (! strpos('@', $uname)) {
                $c = new Criteria();
                $c->add('lower(nick_name)', strtolower($uname));
                $user_by_nick = DB_OzoneUserPeer::instance()->selectOne($c);
                if ($user_by_nick) {
                    $uname = $user_by_nick->getName();
                }
            }
            
    		$user = SecurityManager::authenticateUser($uname, $upass);
		}
		
		if($user == null){
			EventLogger::instance()->logFailedLogin($uname);
			throw new ProcessException(_("The login and password do not match."), "login_invalid");
		}

		$originalUrl = $runData->sessionGet('loginOriginalUrl');
		
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
		
		
		/* If the request is over https:, we should also use loginauth.php script to set non-ssl ip address. */
		
		if($_SERVER['HTTPS']){
			$sessionHash = md5($session->getSessionId() . LoginAuthController::$secretSeed);
			$parms = array('sessionHash' => $sessionHash);
			if($originalUrl){
				$parms['origUrl'] = $originalUrl;
			}
			$originalUrl = 'http://' . GlobalProperties::$URL_HOST. '/loginauth.php?'.http_build_query($parms);
		}
		
		if($originalUrl){
			$runData->ajaxResponseAdd('originalUrl', $originalUrl);
		}
		
		setcookie("welcome", $user->getUserId(), time() + 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);
		setcookie(GlobalProperties::$SESSION_COOKIE_NAME_IE, $runData->getSessionId(), null, "/");
		
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
