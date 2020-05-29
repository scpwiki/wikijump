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

class LoginIframeScreen extends SmartyScreen {
	
	public function build($runData){
		
		$pl = $runData->getParameterList();
		$url = $pl->getParameterValue('url');
		$siteId = $pl->getParameterValue('siteId');
		if($siteId && is_numeric($siteId)){
			$site = DB_SitePeer::instance()->selectByPrimaryKey($siteId);	
		}
		if(!$site){
			throw new ProcessException(_('Invalid site'));	
		}
		
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

		$themeId = $pl->getParameterValue('themeId');
		
		if($themeId && is_numeric($themeId)){
			$theme = DB_ThemePeer::instance()->selectByPrimaryKey($themeId);	
		}
		if(!$theme){
			throw new ProcessException(_('Invalid theme'));	
		}
		
		$runData->contextAdd('site',$site);
		$runData->contextAdd('theme', $theme);
		$runData->contextAdd('url', $url);
		
		$seed = CryptUtils::generateSeed(4);
		
		// put seed into session!
		$runData->sessionStart();
		$runData->sessionAdd("login_seed", $seed);
		
		$runData->contextAdd("key", CryptUtils::modulus());
		$runData->contextAdd("seed", $seed);
		
		// clear welcome cookie?
		if($pl->getParameterValue("clearwelcome")){
			$runData->contextAdd('reset', true);
		}

	}
	
}
