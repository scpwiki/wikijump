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

class ManageSitePrivateSettingsModule extends ManageSiteBaseModule {
	
	protected $processPage = true;
	
	public function build($runData){	
		
		$site = $runData->getTemp("site");
		$runData->contextAdd("site", $site);
		$runData->contextAdd("settings", $site->getSettings());
		$runData->contextAdd("superSettings", $site->getSuperSettings());
		
		// get the viewers
		$c = new Criteria();
		$q = "SELECT ozone_user.* FROM ozone_user, site_viewer WHERE site_viewer.site_id='".$site->getSiteId()."' " .
				"AND ozone_user.user_id = site_viewer.user_id ORDER BY ozone_user.nick_name";
		$c->setExplicitQuery($q);
		
		$viewers = DB_OzoneUserPeer::instance()->select($c);
		
		$runData->contextAdd("viewers", $viewers);
		$runData->contextAdd("settings", $site->getSettings());
		
	}
	
}
