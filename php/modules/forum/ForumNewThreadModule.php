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

class ForumNewThreadModule extends SmartyModule {
	
	private $category;
	protected $processPage = true;

	public function build($runData){
	
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$categoryId = $pl->getParameterValue("c");
		
		if($categoryId == null || !is_numeric($categoryId)){
			throw new ProcessException(_("No forum category has been specified."));	
		}
		
		// check for suggested title
		$sTitle = $pl->getParameterValue("title");
		
		$c = new Criteria();
		$c->add("category_id", $categoryId);
		$c->add("site_id", $site->getSiteId());
		
		$category = DB_ForumCategoryPeer::instance()->selectOne($c);
		
		if($category == null){
			throw new ProcessException(_("No forum category has been specified."));	
		}
		
		WDPermissionManager::instance()->hasForumPermission('new_thread', $runData->getUser(), $category);	
		
		// keep the session - i.e. put an object into session storage not to delete it!!!
		$runData->sessionAdd("keep", true);
		
		$this->category = $category;
		$runData->contextAdd("category", $category);
		
		$userId = $runData->getUserId();
		if($userId == null){
			$userString = $runData->createIpString();	
			$runData->contextAdd("anonymousString", $userString);
		}
		
		if($sTitle){
			$runData->contextAdd("title", $sTitle);	
		}

	}
	
	public function processPage($out, $runData){
		
		if($this->category != null){
			$out = preg_replace("/<title>(.+?)<\/title>/is","<title>\\1 ".preg_quote_replacement(htmlspecialchars($this->category->getName()))."</title>",$out);
			$title = '<a href="/forum/c-'.$this->category->getCategoryId().'/'.htmlspecialchars($this->category->getUnixifiedName()).'">'.htmlspecialchars($this->category->getName()).'</a> / '._('new thread');
		
			$out = preg_replace('/<div id="page-title">(.*?)<\/div>/is','<div id="page-title">'.preg_quote_replacement($title).'</div>',$out);
		}
		return $out;	
	}
	
}
