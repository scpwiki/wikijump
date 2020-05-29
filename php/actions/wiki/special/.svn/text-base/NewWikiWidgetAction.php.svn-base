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

class NewWikiWidgetAction extends SmartyAction{

	public function perform($runData){}

	public function newWikiEvent($runData){
		$pl = $runData->getParameterList();
		
		$siteName = $pl->getParameterValue('siteName');
		
		// validate even more
		$unixName = WDStringUtils::toUnixName($siteName);
			
		if($unixName === null || strlen($unixName)<3){
			throw new ProcessException(_("Web address must be at least 3 characters long."));	
		}
		if(strlen($unixName)>30){
			throw new ProcessException(_("Web address name should not be longer than 30 characters."));
		}
		if(preg_match("/^[a-z0-9\-]+$/", $unixName) == 0){
			throw new ProcessException(_('Only lowercase alphanumeric and "-" (dash) characters allowed in the web address.'));
		}
		if(preg_match("/\-\-/", $unixName) !== 0){
			throw new ProcessException(_('Only lowercase alphanumeric and "-" (dash) characters allowed in the web address. Double-dash (--) is not allowed.'));
		}
		
		$unixName = WDStringUtils::toUnixName($unixName);
		
		if(!$runData->getUser() || !$runData->getUser()->getSuperAdmin()){
		 	//	handle forbidden names	
		 	$forbiddenUnixNames = explode("\n", file_get_contents(WIKIDOT_ROOT.'/conf/forbidden_site_names.conf'));
			foreach($forbiddenUnixNames as $f){
				if(preg_match($f, $unixName) >0){
					throw new ProcessException(_('For some reason this web address is not allowed or is reserved for future use.'));	
				}	
			}
		}

		// check if the domain is not taken.
		$c = new Criteria();
		$c->add("unix_name", $unixName);
		$ss = DB_SitePeer::instance()->selectOne($c);
		if($ss){
			throw new ProcessException(_('Sorry, this web address is already used by another wiki.'));
					
		}	
		
		$runData->ajaxResponseAdd('unixName', $unixName);

	}
	
}
