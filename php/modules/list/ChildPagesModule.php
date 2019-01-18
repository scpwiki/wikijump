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

class ChildPagesModule extends SmartyModule {
	
	public function build($runData){
		$page = $runData->getTemp("page");
		if(!$page){
			$pageName = $runData->getTemp("pageUnixName");
			$site = $runData->getTemp("site");	
			$page =  DB_PagePeer::instance()->selectByName($site->getSiteId(), $pageName);
		}
		
		if(!$page){
			throw new ProcessException(_("Unable to retrieve page data."), "no_page");	
		}
		
		$c = new Criteria();
		$c->add("parent_page_id", $page->getPageId());
		$c->addOrderAscending("COALESCE(title, unix_name)");
		
		$pages = DB_PagePeer::instance()->select($c);
		if(count($pages)>0){
			$runData->contextAdd("pages", $pages);
		}	
	}
	
}
