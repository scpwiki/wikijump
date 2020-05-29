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

class PageTagsModule extends SmartyModule {

	public function build($runData){
		$user = $runData->getUser();
		$pl = $runData->getParameterList();
		$pageId = $pl->getParameterValue("pageId");
		
		$site = $runData->getTemp("site");
		
		if(!$pageId || !is_numeric($pageId)){
			throw new ProcessException(_("The page can not be found or does not exist."), "no_page");	
		}
		
		$page = DB_PagePeer::instance()->selectByPrimaryKey($pageId);
			
		if($page == null || $page->getSiteId() != $site->getSiteId()){
			throw new ProcessException(_("Error getting page information."), "no_page");
		} 
		
		$category = $page->getCategory();
		
		WDPermissionManager::instance()->hasPagePermission('edit', $user, $category, $page);
		
		// get the tags now
		
		$c = new Criteria();
		$c->add("page_id", $pageId);
		
		$c->addOrderAscending("tag");
		
		$tags = DB_PageTagPeer::instance()->select($c);
		
		$t2 = array();
		
		foreach($tags as $t){
			$t2[] = $t->getTag();	
		}	
		
		$t3 = implode(' ', $t2);
		
		$runData->contextAdd("tags", $t3);
	}
	
}
