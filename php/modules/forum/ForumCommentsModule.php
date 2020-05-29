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

class ForumCommentsModule extends SmartyModule {
	
	protected $processPage = true;
	
	public function build($runData){
		$pl = $runData->getParameterList();
		
		// check if forum is activated
		$site = $runData->getTemp("site");
		$fsettings = $site->getForumSettings();
		
		if(!$fsettings){
			throw new ProcessException(_("Forum must be activated for the Comments module to work. Please use the Site Manager."));
		}
		
		$show = true;
		if($pl->getParameterValue("hide", "MODULE")){
			$show = false;	
		}
		// but can be forced via uri parameter "comments"
		if($pl->getParameterValue("comments") == "show"){
			$show = true;	
		}
		
		$pageName = $runData->getTemp("pageUnixName");
		if($pageName == null){
			$show=false;	
		}
		
		$title = $pl->getParameterValue("title","MODULE");
		if($title === null){
			$title = _('Comments');	
		}
		
		$runData->contextAdd("title", $title);
		
		$runData->contextAdd("showComments", $show);
	}
	
	public function processPage($out, $runData){
		$site = $runData->getTemp("site");
		$pageName = $runData->getTemp("pageUnixName");
		if($pageName == null){
			return $out;	
		}
		$page = DB_PagePeer::instance()->selectByName($site->getSiteId(), $pageName);
		$pageId = $page->getPageId();
		$link = '/feed/page/comments-'.$pageId.'.xml';
		$title =  "Comments for the page \"".$page->getTitleOrUnixName()."\"";
		$out = preg_replace("/<\/head>/", 
				'<link rel="alternate" type="application/rss+xml" title="'.htmlspecialchars($title).'" href="'.$link.'"/></head>',
				$out,1);
				
		return $out;
	}
	
}
