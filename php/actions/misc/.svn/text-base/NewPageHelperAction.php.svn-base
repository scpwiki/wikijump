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

class NewPageHelperAction extends SmartyAction {
	
	public function perform($r){}
	
	public function createNewPageEvent($runData){
		// this just checks if page exists and if the user has permissions to create.
		// returns cleaned name.
		
		$pl = $runData->getParameterList();
		$pageName = trim($pl->getParameterValue("pageName"));
		$categoryName = trim($pl->getParameterValue("categoryName"));
		$format =  trim($pl->getParameterValue("format"));
		$autoincrement = $pl->getParameterValue('autoincrement');
		
		$templateId = $pl->getParameterValue("template");
		
		$site = $runData->getTemp("site");
		
		if(strlen($pageName) === 0){
			$runData->ajaxResponseAdd("status", "no_name");
			$runData->ajaxResponseAdd("message", "You should provide a page name.");
			return;	
		}

		// check if use a title too
		//if(WDStringUtils::toUnixName($pageName) != $pageName){
			$pageTitle = $pageName;	
		//}
		
		if($format){
			$m = false;
			$m = @preg_match($format, $pageName);

			if($m !== false && $m === 0){
				throw new ProcessException(_("The page name is not in the required format."));	
			}
		}
		
		if($autoincrement){
			$unixName = $categoryName . ':autoincrementpage';
		} else {
			$unixName = WDStringUtils::toUnixName($categoryName.':'.$pageName);
		}
		
		$page = DB_PagePeer::instance()->selectByName($site->getSiteId(), $unixName);
		if($page != null){
			$runData->ajaxResponseAdd("status", "page_exists");
			$runData->ajaxResponseAdd("message", "The page <em>".$unixName."</em> already exists." .
					' <a href="/'.$unixName.'">Jump to it</a> if you wish.');
			return;		
		}

		if($templateId){
			
			$templatePage = DB_PagePeer::instance()->selectByPrimaryKey($templateId);
			if(!$templatePage || !preg_match("/^template:/", $templatePage->getUnixName())){
				throw new ProcessException("Error selecting the template");	
			}
			
			$runData->ajaxResponseAdd("templateId", $templateId);
		}

		$runData->ajaxResponseAdd("unixName", $unixName);
		if($pageTitle){
			$runData->ajaxResponseAdd("pageTitle", $pageTitle);	
		}
	}
	
}
