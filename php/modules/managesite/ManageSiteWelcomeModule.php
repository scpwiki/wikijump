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

class ManageSiteWelcomeModule extends ManageSiteBaseModule {
	
	public function build($runData){
		
		$site = $runData->getTemp("site");
		
		$fsettings = $site->getForumSettings();
		
		$tips = array();
		
		if(!$fsettings){
			$tips['forum'] = true;
		}
		
		// site tags
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$t = DB_SiteTagPeer::instance()->selectOne($c);
		
		if(!$t){
			$tips['tags'] = true;	
		}
		
		// count members... ???
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$co = DB_MemberPeer::instance()->selectCount($c);
		
		if($co<4){
			$tips['invite'] = true;	
		}

		if(count($tips)>0){
			$runData->contextAdd("tips", $tips);	
		}
		
		$runData->contextAdd('site', $site);

	}
	
}
