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

class ManageSiteAnonymousAbuseModule extends ManageSiteBaseModule {
	
	public function build($runData){
		
		$site = $runData->getTemp("site");
		
		// get 
		$q = "SELECT address, proxy, count(*) AS rank " .
				"FROM anonymous_abuse_flag " .
				"WHERE site_id='".$site->getSiteId()."' " .
				"AND site_valid = TRUE GROUP BY address, proxy ORDER BY rank DESC, address";

		$db = Database::connection();
		$res = $db->query($q);
		
		$all = $res->fetchAll();
		
		$runData->contextAdd("reps", $all);

	}

}
