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

class FeedFlowController extends WebFlowController {

	public function process() {

		// initialize logging service
		$logger = OzoneLogger::instance();
		$loggerFileOutput = new OzoneLoggerFileOutput();
		$loggerFileOutput->setLogFileName(WIKIDOT_ROOT."/logs/ozone.log");
		$logger->addLoggerOutput($loggerFileOutput);
		$logger->setDebugLevel(GlobalProperties::$LOGGER_LEVEL);
		
		$logger->debug("Feed request processing started, logger initialized");

		Ozone ::init();
		
		$runData = new RunData();
		$runData->init();
		Ozone :: setRunData($runData);
		$logger->debug("RunData object created and initialized");

		// check if site (wiki) exists!
		$siteHost = $_SERVER["HTTP_HOST"];

		$memcache = Ozone::$memcache;
		if(preg_match("/^([a-zA-Z0-9\-]+)\." . GlobalProperties::$URL_DOMAIN . "$/", $siteHost, $matches)==1){
			$siteUnixName=$matches[1];
			// select site based on the unix name
			
			// check memcached first!
			
			// the memcache block is to avoid database connection if possible
			
			$mcKey = 'site..'.$siteUnixName;
			$site = $memcache->get($mcKey); 
			if($site == false){
				$c = new Criteria();
				$c->add("unix_name", $siteUnixName);
				$c->add("site.deleted", false);
				$site = DB_SitePeer::instance()->selectOne($c);
				$memcache->set($mcKey, $site, 0, 3600);	
			}
		} else {
			// select site based on the custom domain
			$mcKey = 'site_cd..'.$siteHost;
			$site = $memcache->get($mcKey);
			if($site == false){	
				$c = new Criteria();
				$c->add("custom_domain", $siteHost);
				$c->add("site.deleted", false);
				$site = DB_SitePeer::instance()->selectOne($c);
				$memcache->set($mcKey, $site, 0, 3600);	
			}
			GlobalProperties::$SESSION_COOKIE_DOMAIN = '.'.$siteHost;
			
		}
		
		if($site == null){
			$content = file_get_contents(WIKIDOT_ROOT."/files/site_not_exists.html");
			echo $content;
			return $content;	
		} 
		
		$runData->setTemp("site", $site);	
		//nasty global thing...
		$GLOBALS['siteId'] = $site->getSiteId();
		$GLOBALS['site'] = $site;

		// set language
		$lang = $site->getLanguage();
		$runData->setLanguage($lang);
		$GLOBALS['lang'] = $lang;
		
		// and for gettext too:
		
		switch($lang){
			case 'pl':
				$glang="pl_PL";
				break;
			case 'en':
				$glang="en_US";
				break;
		}

		putenv("LANG=$glang"); 
		putenv("LANGUAGE=$glang"); 
		setlocale(LC_ALL, $glang.'.UTF-8');
		
		$settings = $site->getSettings();
		// handle SSL 
		$sslMode = $settings->getSslMode();
		if($_SERVER['HTTPS']){
			if(!$sslMode){
				// not enabled, redirect to http:
				echo _("Secure access is not enabled for this Wiki.");
				exit;
			}
		}

		$template = $runData->getScreenTemplate();
		$classFile = $runData->getScreenClassPath();
		$className = $runData->getScreenClassName();
		$logger->debug("processing template: ".$runData->getScreenTemplate().", class: $className");

		require_once ($classFile);
		$screen = new $className ();
			
		// check if requires authentication
		if($screen->getRequiresAuthentication() || $site->getPrivate()){
			$username = $_SERVER['PHP_AUTH_USER'];
			$password = $_SERVER['PHP_AUTH_PW'];
			$user = null;
			if($username !== null && $password !== null){
				$user = SecurityManager::getUserByName($username);
				if($user){
					$upass = md5("feed_hashed_password_".$user->getPassword());
					$upass = substr($upass,0,15);
					if($upass !== $password){
						$user = null;
					}
				}	
			}	
			
			if($site->getPrivate()){
				if($user && !$user->getSuperAdmin() && !$user->getSuperModerator()){
					// check if member
					$c = new Criteria();
					$c->add("site_id", $site->getSiteId());
					$c->add("user_id", $user->getUserId());
					$mem = DB_MemberPeer::instance()->selectOne($c);
					if(!$mem) { 
						// check if a viewer
						$c = new Criteria();
						$c->add("site_id", $site->getSiteId());
						$c->add("user_id", $user->getUserId());
						$vi = DB_SiteViewerPeer::instance()->selectOne($c);
						if(!$vi) { 
							$user = null;
						}
					}
				}
			}
			
			if($user == null){
				header('WWW-Authenticate: Basic realm="Private"');
   				header('HTTP/1.0 401 Unauthorized');
   				header('Content-type: text/plain; charset=utf-8');
   				echo _("This is a private feed. User authentication required via Basic HTTP Authentication. You can not access it. Please go to 'Account settings' -> 'Notifications' to get the password if you believe you should be allowed.");
   				exit();
			}
			$runData->setTemp("user", $user);
		}	
	
		$logger->debug("OZONE initialized");

		$logger->info("Ozone engines successfully initialized");

		$rendered = $screen->render($runData);

		echo str_replace("%%%CURRENT_TIMESTAMP%%%",time(),$rendered);
		
		return $rendered;
	}

}
