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

class ListAllWikisModule extends CacheableModule{
	
	protected $timeOut=30;
	
	public function build($runData){
		$pl = $runData->getParameterList();
		$categoryId = $pl->getParameterValue("c");
		
		$pageNumber = $pl->getParameterValue("p");
		if($pageNumber == null || !is_numeric($pageNumber) || $pageNumber <1){
			$pageNumber = 1;	
		}
		
		$sort = $pl->getParameterValue("sort");
		
		// the criteria is: have >= 20 edits.
		
		// first - count them all
		//$q = 
		
		$c = new Criteria();

		$q = "SELECT site.* FROM site WHERE  site.visible = TRUE AND site.private = FALSE AND site.deleted = FALSE AND site.site_id != 1 AND (SELECT count(*) FROM page WHERE page.site_id = site.site_id) > 15 ORDER BY site.name";
		
		$c->setExplicitQuery($q);
		
		$sites = DB_SitePeer::instance()->select($c);
		
		$runData->contextAdd("sites", $sites);
		
	}
	
}
