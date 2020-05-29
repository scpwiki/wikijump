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

class RenamePageModule extends SmartyModule {
	
	public function build($runData){
		// only check for permissions
		$pl = $runData->getParameterList();
		$pageId = $pl->getParameterValue("pageId");
		$page = DB_PagePeer::instance()->selectByPrimaryKey($pageId);
		if($page == null || $page->getSiteId() != $runData->getTemp("site")->getSiteId()){
			throw new ProcessException(_("Error getting page information."), "no_page");
		}	
		
		$delete = $pl->getParameterValue("delete");
		
		$user = $runData->getUser();
		
		if($delete){
			$newName = 'deleted:'.$page->getUnixName();
			$runData->contextAdd("delete", true);
		}else{
			$newName = $page->getUnixName();
		}
		
		$category = $page->getCategory();
		$runData->contextAdd("page", $page);
		
		$runData->contextAdd("newName", $newName);
		
		// now check for permissions!!!
		
		WDPermissionManager::instance()->hasPagePermission('move', $user, $category, $page);
		
		$canDelete = true;
		try{
			WDPermissionManager::instance()->hasPagePermission('delete', $user, $category, $page);
		}catch(Exception $e){
			$canDelete = false;	
		}
		
		$runData->contextAdd("canDelete", $canDelete);
		
		// check if belongs to a special category...
		$categoryName = $category->getName();
		if($categoryName == "forum"){
			$runData->contextAdd("isForum", true);	
		}
		if($categoryName == "admin"){
			$runData->contextAdd("isAdmin", true);	
		}
	}
	
}
