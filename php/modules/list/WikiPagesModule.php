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

class WikiPagesModule extends CacheableModule {
	
	protected $timeOut = 10;
	
	public function build($runData){
		
		$pl = $runData->getParameterList();
		$site = $runData->getTemp("site");
		
		$categoryName = $pl->getParameterValue("category", "MODULE", "AMODULE");
		$details = $pl->getParameterValue("details", "MODULE", "AMODULE");
		$preview = $pl->getParameterValue("preview", "MODULE", "AMODULE");
		
		$order = $pl->getParameterValue("order", "MODULE", "AMODULE");
		$limit = $pl->getParameterValue("limit", "MODULE", "AMODULE");
		
		if($categoryName !== null){
			$category = DB_CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId());
			if($category == null){
				throw new ProcessException(_("The category can not be found."));	
			}
		}
		
		// now select pages according to the specified criteria
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		if($category){
			$c->add("category_id", $category->getCategoryId());	
		}
		
		switch($order){
			case 'dateCreatedDesc':
				$c->addOrderDescending('page_id');
				break;
			case 'dateCreatedAsc':
				$c->addOrderAscending('page_id');
				break;
			case 'dateEditedDesc':
				$c->addOrderDescending('date_last_edited');
				break;
			case 'dateEditedAsc':
				$c->addOrderAscending('date_last_edited');
				break;
			case 'titleDesc':
				$c->addOrderDescending("COALESCE(title, unix_name)");
				break;
			default:
				$c->addOrderAscending("COALESCE(title, unix_name)");
		}
		
		if($limit && is_numeric($limit) && $limit > 0){
			$c->setLimit($limit);	
		}
		
		$pages = DB_PagePeer::instance()->select($c);
		
		// by default cathegorize by first letter...
		
		$runData->contextAdd("pages", $pages);
		$runData->contextAdd("details", $details);
		$runData->contextAdd("preview", $preview);
		
	}
	
}
