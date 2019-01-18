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

class NewSiteAction extends SmartyAction {
	
	public function isAllowed($runData){
		if($runData->getUser() == null){
			throw new WDPermissionException(_("You should be logged in to create a new site."));	
		}	
		return true;
	}
	
	public function perform($runData){}
	
	public function createSiteEvent($runData){
		
		WDPermissionManager::instance()->canBecomeAdmin($runData->getUser());
		
		$pl = $runData->getParameterList();
		
		$name = trim($pl->getParameterValue("name"));
		$unixName = trim($pl->getParameterValue("unixname"));
		$tagline = trim($pl->getParameterValue("tagline"));
		
		$templateId = $pl->getParameterValue("template");
		
		$private = (bool) $pl->getParameterValue("private");
	
		// validate form data:
		
		$errors = array();
		if(strlen($name)<1){
			$errors['name'] = _("Site name must be present.");	
		}elseif(strlen8($name)>30){
			$errors['name']	 = _("Site name should not be longer than 30 characters.");
		}
		
		// site unix name *************
		if($unixName === null || strlen($unixName)<3){
			$errors['unixname'] = _("Web address must be present and should be at least 3 characters long.");	
		}elseif(strlen($unixName)>30){
			$errors['unixname']	 = _("Web address name should not be longer than 30 characters.");
		}elseif(preg_match("/^[a-z0-9\-]+$/", $unixName) == 0){
			$errors['unixname']	= _('Only lowercase alphanumeric and "-" (dash) characters allowed in the web address.');
		}elseif(preg_match("/\-\-/", $unixName) !== 0){
			$errors['unixname']	= _('Only lowercase alphanumeric and "-" (dash) characters allowed in the web address. Double-dash (--) is not allowed.');
		}else{
			$unixName = WDStringUtils::toUnixName($unixName);
			
			if(!$runData->getUser()->getSuperAdmin()){
			 	//	handle forbidden names	
			 	$forbiddenUnixNames = explode("\n", file_get_contents(WIKIDOT_ROOT.'/conf/forbidden_site_names.conf'));
				foreach($forbiddenUnixNames as $f){
					if(preg_match($f, $unixName) >0){
						$errors['unixname']	= _('For some reason this web address is not allowed or is reserved for future use.');	
					}	
				}
			}
		
			// check if the domain is not taken.
			$c = new Criteria();
			$c->add("unix_name", $unixName);
			$ss = DB_SitePeer::instance()->selectOne($c);
			if($ss){
				$errors['unixname'] = _('Sorry, this web address is already used by another site.');		
			}			
		}

		// template
		if(!$templateId){
			$errors['template']	= _('Please choose a template for your site');
		}

		if(strlen8($tagline)>50){
			$errors['tagline']	 = _("Tagline should not be longer than 50 characters");
		}

		// TOS
		if(!$pl->getParameterValue("tos")){
			$errors['tos'] = _("Please read and agree to the Terms of Service.");	
		}

		if(count($errors)>0){
			$runData->ajaxResponseAdd("formErrors", $errors);	
			throw new ProcessException("Form errors", "form_errors");
		}
		
		// and now... CREATE THE SITE!!!!!!!!!!!!!!!!
		
		$dup = new Duplicator();
		$dup->setOwner($runData->getUser());
		
		$db = Database::connection();
		$db->begin();

		$templateSite = DB_SitePeer::instance()->selectByPrimaryKey($templateId);
		if(!preg_match(';^template\-;', $templateSite->getUnixName())){
		    throw new ProcessException('Error');
		}
		
		$site = new DB_Site();
		$site->setName($name);
		$site->setSubtitle($tagline);
		$site->setUnixName($unixName);
		$site->setLanguage($templateSite->getLanguage());
		$site->setDateCreated(new ODate());
		
		$site->setPrivate($private);
		
		if($private){
			// change file flag too
			$flagDir = WIKIDOT_ROOT.'/web/files--sites/'.$site->getUnixName().'/flags';
			$flagFile = $flagDir.'/private';
			mkdirfull($flagDir); //just to make sure
			
			if(!file_exists($flagFile)){
				file_put_contents($flagFile, "private");
			}	
		}
		
		$site->save();
		
		$dup->addExcludedCategory("forum"); // should be initialized independently
		$dup->addExcludedCategory("profile");
		$dup->duplicateSite($templateSite, $site);
		
		// index the site too
		$ind = Indexer::instance();
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$pages = DB_PagePeer::instance()->select($c);
		foreach($pages as $p){
			$ind->indexPage($p);
		}	
	
		$db->commit();	
		
		// clear captcha code
		$runData->sessionDel("captchaCode");
		
		$runData->ajaxResponseAdd("siteUnixName", $unixName);
	}
	
}
