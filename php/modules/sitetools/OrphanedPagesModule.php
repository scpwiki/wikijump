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

class OrphanedPagesModule extends SmartyModule {
	
	public function build($runData){
		$site = $runData->getTemp("site");
		$siteId = $site->getSiteId();
		
		$q = "SELECT *, count(*) AS number_links FROM page, page_link " .
				"WHERE page.site_id = '$siteId' AND page_link.to_page_id=page.page_id " .
				"GROUP BY (page.page_id) " .
				"ORDER BY COALESCE(page.title, page.unix_name)";
		
		$q = "SELECT * FROM page " .
				"WHERE page.site_id = '$siteId'" .
				"AND (SELECT count(*) FROM page_link WHERE page_link.to_page_id = page.page_id) = 0 ".
				"ORDER BY COALESCE(page.title, page.unix_name)";
		
		$c = new Criteria();
		$c->setExplicitQuery($q);
		
		$pages = DB_PagePeer::instance()->select($c);
		
		$runData->contextAdd("pages", $pages);
		
		
			
	}
	
}
