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

class PageBlockModule extends SmartyModule {

	public function build($runData){
	
		$pl = $runData->getParameterList();
		$site = $runData->getTemp("site");
		
		$pageId = $pl->getParameterValue("page_id");
		$user = $runData->getUser();

		$page = DB_PagePeer::instance()->selectByPrimaryKey($pageId);
		if(!$pageId || $page == null || $page->getSiteId() != $runData->getTemp("site")->getSiteId()){
			throw new ProcessException(_("Error getting page information."), "no_page");
		}	
		
		if($this->canSetBlock($user, $page) == false){
			throw new WDPermissionException(_("Sorry, only Site Admnistrators and selected Moderators can block a page."));	
		}
		
		$runData->contextAdd("page", $page);

	}
	
	private function canSetBlock($user, $page){
		
		if($user && ($user->getSuperAdmin() || $user->getSuperModerator())){
			return true;	
		}
		
		if(!$user){
			return false;	
		}
		
		// still nothing. check if moderator of "pages".
		$c = new Criteria();
		$c->add("site_id", $page->getSiteId());
		$c->add("user_id", $user->getUserId());
		$rel = DB_ModeratorPeer::instance()->selectOne($c);
		if($rel && strpos($rel->getPermissions(), 'p') !== false){
			return true;
		}
			
		// still nothing. check if admin.
		$c = new Criteria();
		$c->add("site_id", $page->getSiteId());
		$c->add("user_id", $user->getUserId());
		$rel = DB_AdminPeer::instance()->selectOne($c);
		if($rel){
			return true;
		}
		
		return false;
	}
	
}
