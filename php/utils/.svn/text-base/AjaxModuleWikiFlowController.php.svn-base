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

class AjaxModuleWikiFlowController extends WebFlowController {

	public function process() {
		global $timeStart;

		// initialize logging service
		$logger = OzoneLogger::instance();
		$loggerFileOutput = new OzoneLoggerFileOutput();
		$loggerFileOutput->setLogFileName(WIKIDOT_ROOT."/logs/ozone.log");
		$logger->addLoggerOutput($loggerFileOutput);
		$logger->setDebugLevel(GlobalProperties::$LOGGER_LEVEL);
		
		$logger->debug("AJAX module request processing started, logger initialized");

		Ozone ::init();
		
		$runData = new RunData();
		/* processing an AJAX request! */
		$runData->setAjaxMode(true);
		
		$runData->init();
		
		// extra return array - just for ajax handling
		$runData->ajaxResponseAdd("status", "ok");

		Ozone :: setRunData($runData);
		$logger->debug("RunData object created and initialized"); 

		try{
			
			// check security token
			if($_COOKIE['wikidot_token7'] == null || $_COOKIE['wikidot_token7'] !== $runData->getParameterList()->getParameterValue('wikidot_token7','AMODULE')){
				throw new ProcessException("no", "wrong_token7");	
			}
			//remove token from parameter list!!!
			$runData->getParameterList()->delParameter('wikidot_token7');
			
			$callbackIndex = $runData->getParameterList()->getParameterValue('callbackIndex');
			$runData->getParameterList()->delParameter('callbackIndex');
			
			// check if site (wiki) exists!
			$siteHost = $_SERVER["HTTP_HOST"];

			$memcache = Ozone::$memcache;
			if(preg_match("/^([a-zA-Z0-9\-]+)\." . GlobalProperties::$URL_DOMAIN_PREG . "$/", $siteHost, $matches)==1){
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
			 
			if(!$site){
				throw new ProcessException(_('The requested site does not exist.'));
			}
			
			$runData->setTemp("site", $site);	
			//nasty global thing...
			$GLOBALS['siteId'] = $site->getSiteId();
			$GLOBALS['site'] = $site;
			
			// set language
			$runData->setLanguage($site->getLanguage());
			$GLOBALS['lang'] = $site->getLanguage();
			
			// and for gettext too:
		
			$lang = $site->getLanguage();
		
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
					// not enabled, issue an errorr
					throw new ProcessException(_("Secure access is not enabled for this Wiki."));
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
						throw new ProcessException(_("Nonsecure access is not enabled for this Wiki."));
						break;
					
				}
			}
		
			// handle session at the begging of procession
			$runData->handleSessionStart();

			// PRIVATE SITES: check if the site is private and if the user is its member
			
			if($site->getPrivate()){
				// check if not allow anyway
				$template = $runData->getModuleTemplate();
				$actionClass = $runData->getAction();

				$proceed = in_array($actionClass, array('', 'LoginAction', 'MembershipApplyAction', 'CreateAccountAction', 'PasswordRecoveryAction'))
						&& ($template == '' || $template == 'Empty' || preg_match(';^createaccount/;', $template) || preg_match(';^login/;',$template) || preg_match(';^membership/;',$template)
						|| preg_match(';^passwordrecovery/;',$template));
				if(!$proceed){
					$user = $runData->getUser();
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
					if($user == null){
						throw new ProcessException(_('This Site is private and accessible only to its members.'));
					}	
				}
			}

			$template = $runData->getModuleTemplate();
			$classFile = $runData->getModuleClassPath();
			$className = $runData->getModuleClassName();
			$logger->debug("processing template: ".$runData->getModuleTemplate().", class: $className");
	
			require_once ($classFile);
			$module = new $className ();

			// module security check
			if(!$module->isAllowed($runData)){
				throw new WDPermissionException(_("Not allowed."));
			}

			Ozone::initSmarty();
			$logger->debug("OZONE initialized");
			
			$logger->info("Ozone engines successfully initialized");

			// PROCESS ACTION
			$actionClass = $runData->getAction();
			$logger->debug("processing action $actionClass");
			
			$runData->setTemp("jsInclude", array());
			$runData->setTemp("cssInclude", array());
			
			if($actionClass){
				
				require_once (PathManager :: actionClass($actionClass));
				$tmpa1 = explode('/', $actionClass);
		        $actionClassStripped = end($tmpa1);
		
				$action = new $actionClassStripped();
					
				$classFile = $runData->getModuleClassPath();
				if(!$action->isAllowed($runData)){
					throw new WDPermissionException("Not allowed.");
						
				}
				
				$actionEvent = $runData->getActionEvent();
				/*try{*/
					if ($actionEvent != null) {
						$action-> $actionEvent ($runData);
						$logger->debug("processing action: $actionClass, event: $actionEvent");
					} else {
						$logger->debug("processing action: $actionClass");
						$action->perform($runData);
					}
			}
	
			// end action process
		
			// check if template has been changed by the module. if so...
			if($template != $runData->getModuleTemplate()){
				
				$classFile = $runData->getModuleClassPath();
				$className = $runData->getModuleClassName();
				$logger->debug("processing template: ".$runData->getModuleTemplate().", class: $className");
	
				require_once ($classFile);
				$module = new $className ();
			}

			$module->setTemplate($template);

			$rendered = $module->render($runData);
			
			$jsInclude = $runData->getTemp("jsInclude");
			$jsInclude = array_merge($jsInclude, $module->getExtraJs());
			$runData->setTemp("jsInclude",$jsInclude);
			
			$cssInclude = $runData->getTemp("cssInclude");
			$cssInclude = array_merge($cssInclude, $module->getExtraCss());
			$runData->setTemp("cssInclude",$cssInclude);
			
		}catch(ProcessException $e){
			$db = Database::connection();
			$db->rollback();
			$runData->ajaxResponseAdd("message",$e->getMessage());
			$runData->ajaxResponseAdd("status", $e->getStatus());
			$runData->setModuleTemplate(null);	
			$template=null;
		}catch(WDPermissionException $e){
				$db = Database::connection();
				$db->rollback();
				$runData->ajaxResponseAdd("message",$e->getMessage());
				$runData->ajaxResponseAdd("status", "no_permission");
				$runData->setModuleTemplate(null);	
				$template=null;
		}catch(Exception $e){
			$db = Database::connection();
			$db->rollback();
			$runData->ajaxResponseAdd("message",_("An error occured while processing the request.").' '.$e->getMessage());
			$runData->ajaxResponseAdd("status", "not_ok");
			$runData->setModuleTemplate(null);	
			$template=null;
			// LOG ERROR TOO!!!
			$logger = OzoneLogger::instance();
			$logger->error("Exception caught while processing ajax module:\n\n".$e->__toString());
		}
		
		$rVars = $runData->getAjaxResponse();
		
		if ($rendered != null) {
			// process modules...
	 		$moduleProcessor = new ModuleProcessor($runData);
	 		$out = $moduleProcessor->process($rendered);
	 		$rVars['body'] = $out;

	 		// check the javascript files for inclusion
	 		
		}

		if($template != null && $template != "Empty"){
			$jsInclude = $runData->getTemp("jsInclude");
			if($module->getIncludeDefaultJs()){
		 		$file = WIKIDOT_ROOT.'/'.GlobalProperties::$MODULES_JS_PATH.'/'.$template.'.js';
		 		if(file_exists($file)){
					$url = 	GlobalProperties::$MODULES_JS_URL.'/'.$template.'.js';
					$incl = $url;
					$jsInclude[] = $incl;
				}
			}
			$rVars['jsInclude'] = $jsInclude;
			
			$cssInclude = $runData->getTemp("cssInclude");
			if($module->getIncludeDefaultCss()){
		 		$file = WIKIDOT_ROOT.'/'.GlobalProperties::$MODULES_CSS_PATH.'/'.$template.'.css';
		 		if(file_exists($file)){
					$url = 	GlobalProperties::$MODULES_CSS_URL.'/'.$template.'.css';
					$incl = $url;
					$cssInclude[] = $incl;
				}
			}
			$rVars['cssInclude'] = $cssInclude;
			
		}
		
		// specify (copy) jscallback. ugly, right? ;-)
	 	$rVars['callbackIndex'] = $callbackIndex;
	 		
		$json = new JSONService();
	 	$out = $json->encode($rVars);
	 	
		$runData->handleSessionEnd();
	
		echo $out;
	}
}
