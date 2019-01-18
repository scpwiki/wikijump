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

class AWPagesListModule extends AccountBaseModule {

	public function build($runData){
		
		$user = $runData->getUser();
		$runData->contextAdd("user",$user);	
		
		$pl = $runData->getParameterList();
		
		// get watched pages for this user
		
		$c = new Criteria();
		
		$q = "SELECT page.* FROM watched_page, page " .
				"WHERE watched_page.user_id='".$user->getUserId()."' " .
						"AND watched_page.page_id=page.page_id";
		$c->setExplicitQuery($q);	
				
		$pages = DB_PagePeer::instance()->select($c);
		
		$runData->contextAdd("pages", $pages);
		
		$runData->contextAdd("pagesCount", count($pages));
		
	}

}
