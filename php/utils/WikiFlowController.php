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

class WikiFlowController extends WebFlowController {

	public function process() {
		global $timeStart;
		
		// quick fix to prevent recursive RSS access by Wikidot itself.
		if(strpos($_SERVER['HTTP_USER_AGENT'], 'MagpieRSS') !== false){
			exit();
		}
		
		// initialize logging service
		$logger = OzoneLogger::instance();
		$loggerFileOutput = new OzoneLoggerFileOutput();
		$loggerFileOutput->setLogFileName(WIKIDOT_ROOT."/logs/ozone.log");
		$logger->addLoggerOutput($loggerFileOutput);
		$logger->setDebugLevel(GlobalProperties::$LOGGER_LEVEL);
		
		$logger->debug("request processing started, logger initialized");

		Ozone ::init();
		
		$runData = new RunData();
		$runData->init();
		Ozone :: setRunData($runData);
		$logger->debug("RunData object created and initialized"); 

		// check if site (wiki) exists!
		$siteHost = $_SERVER["HTTP_HOST"];

		$memcache = Ozone::$memcache;
		if(preg_match("/^([a-zA-Z0-9\-]+)\." . GlobalProperties::$URL_DOMAIN_PREG . "$/", $siteHost, $matches)==1){
			$siteUnixName=$matches[1];

			// select site based on the unix name
			
			// check memcached first!
			
			$mcKey = 'site..'.$siteUnixName;
			$site = $memcache->get($mcKey); 

			if(!$site){
				$c = new Criteria();
				$c->add("unix_name", $siteUnixName);
				$c->add("site.deleted", false);
				$site = DB_SitePeer::instance()->selectOne($c);
				if($site) {$memcache->set($mcKey, $site, 0, 864000);}	
			}
		} else {
			// select site based on the custom domain
			$mcKey = 'site_cd..'.$siteHost;
			$site = $memcache->get($mcKey);

			if(!$site){	
				$c = new Criteria();
				$c->add("custom_domain", $siteHost);
				$c->add("site.deleted", false);
				$site = DB_SitePeer::instance()->selectOne($c);
				if($site) {$memcache->set($mcKey, $site, 0, 3600);}	
			}
			
			if(!$site){
				// check for redirects
				$c = new Criteria();
				$q = "SELECT site.* FROM site, domain_redirect WHERE domain_redirect.url='".db_escape_string($siteHost)."' " .
						"AND site.deleted = false AND site.site_id = domain_redirect.site_id LIMIT 1";
				$c->setExplicitQuery($q);
				$site = DB_SitePeer::instance()->selectOne($c);
				if($site){
					$newUrl = 'http://'.$site->getDomain().$_SERVER['REQUEST_URI'];
					header("HTTP/1.1 301 Moved Permanently");
					header("Location: ".$newUrl);
					exit();	
				}
			}
			
			GlobalProperties::$SESSION_COOKIE_DOMAIN = '.'.$siteHost;
			
		}

		if(!$site){
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

		// Set the text domain as 'messages'
		$gdomain = 'messages';
		bindtextdomain($gdomain, WIKIDOT_ROOT.'/locale'); 
		textdomain($gdomain);

		$settings = $site->getSettings();
		// handle SSL 
		$sslMode = $settings->getSslMode();
		if($_SERVER['HTTPS']){
			if(!$sslMode){
				// not enabled, redirect to http:
				header("HTTP/1.1 301 Moved Permanently");
				header("Location: ".'http://'.$_SERVER["HTTP_HOST"].$_SERVER['REQUEST_URI']);
				exit();	
			}elseif($sslMode == "ssl_only_paranoid"){
				// use secure authentication cookie
				// i.e. change authentication scheme
				GlobalProperties::$SESSION_COOKIE_NAME = "WIKIDOT_SESSION_SECURE_ID";
				GlobalProperties::$SESSION_COOKIE_SECURE = true;
				
			}
			
		}else{
			// page accessed via http (nonsecure)
			switch($sslMode){
				case 'ssl':
					//enabled, but nonsecure allowed too.
					break;
				case 'ssl_only_paranoid':
				case 'ssl_only':
					header("HTTP/1.1 301 Moved Permanently");
					header("Location: ".'https://'.$_SERVER["HTTP_HOST"].$_SERVER['REQUEST_URI']);
					exit();	
					break;
				
			}
		}

		// handle session at the begging of procession
		$runData->handleSessionStart();

		$template = $runData->getScreenTemplate();
		$classFile = $runData->getScreenClassPath();
		$className = $runData->getScreenClassName();
		$logger->debug("processing template: ".$runData->getScreenTemplate().", class: $className");

		require_once ($classFile);
		$screen = new $className ();

		$logger->debug("OZONE initialized");

		$logger->info("Ozone engines successfully initialized");

		$rendered = $screen->render($runData);

		if ($rendered != null) {
			$runData->setTemp("jsInclude", array());
			// process modules...
	 		$moduleProcessor = new ModuleProcessor($runData);
	 		//$moduleProcessor->setJavascriptInline(true); // embed associated javascript files in <script> tags
	 		$moduleProcessor->setCssInline(true);
	 		$rendered = $moduleProcessor->process($rendered);
	 		
	 		$jss = $runData->getTemp("jsInclude");
	 		
	 		$jss = array_unique($jss);
	 		$incl = '';
	 		foreach($jss as $js){
	 			$incl .= '<script type="text/javascript" src="'.$js.'"></script>';	
	 		}
	 		$rendered = preg_replace(';</head>;', $incl.'</head>', $rendered);
			
		}

		$runData->handleSessionEnd();

		// one more thing - some url will need to be rewritten if using HTTPS
		if($_SERVER['HTTPS']){
			// ?
			// scripts
			$rendered = preg_replace(';<script(.*?)src="http://' . GlobalProperties::$URL_HOST_PREG . '(.*?)</script>;s', '<script\\1src="https://' . GlobalProperties::$URL_HOST . '\\2</script>', $rendered);
			$rendered = preg_replace(';<link(.*?)href="http://' . GlobalProperties::$URL_HOST_PREG . '(.*?)/>;s', '<link\\1href="https://' . GlobalProperties::$URL_HOST . '\\2/>', $rendered);
			$rendered = preg_replace(';(<img\s+.*?src=")http(://' . GlobalProperties::$URL_HOST_PREG . '(.*?)/>);s', '\\1https\\2', $rendered);
			do{
				$renderedOld = $rendered;
				$rendered = preg_replace(';(<style\s+[^>]*>.*?@import url\()http(://' . GlobalProperties::$URL_HOST_PREG . '.*?</style>);si', '\\1https\\2', $rendered);

			}while($renderedOld != $rendered);
		}

		if (GlobalProperties::$SEARCH_HIGHLIGHT) {
			$rendered = Wikidot_Search_Highlighter::highlightIfSuitable($rendered, $_SERVER["REQUEST_URI"], $_SERVER["HTTP_REFERER"]);
		}
		
		echo str_replace("%%%CURRENT_TIMESTAMP%%%",time(),$rendered);
		
		return $rendered;
	}
	
	private function _fixHttpsStyles($matches){

	}

}
