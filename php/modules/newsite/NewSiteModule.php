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

class NewSiteModule extends SmartyModule {
	
	public function build($runData){
		if($runData->getUser() == null){
			$runData->contextAdd("notLogged", true);	
		}else{
			
//			
//			
		}
		$pl = $runData->getParameterList();
		$siteUnixName = WDStringUtils::toUnixName($pl->getParameterValue('address'));
		$runData->contextAdd('unixName', $siteUnixName);
		
		$siteName = str_replace('-', ' ', $siteUnixName);
		$siteName = ucwords($siteName);	
		$runData->contextAdd('siteName', $siteName);	
		
		// get template sites
		$c = new Criteria();
		$c->add('unix_name', '^template-', '~');
		$c->addOrderAscending('site_id');
		$templates = DB_SitePeer::instance()->select($c);
		$runData->contextAdd('templates', $templates);
	}
	
}
