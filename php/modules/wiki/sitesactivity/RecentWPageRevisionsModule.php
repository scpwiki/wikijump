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

class RecentWPageRevisionsModule extends SmartyModule {
	
	public function render($runData){
		$site = $runData->getTemp("site");
		$key = "module..0..RecentWPageRevisionsModule..".$site->getSiteId().'..'.md5(serialize($runData->getParameterList()->asArray()));
		$mc = OZONE::$memcache;
		
		$out = $mc->get($key);
		if(!$out){
			$out = parent::render($runData);
			$mc->set($key, $out, 0, 120);	
		} 
		
		return $out;
		
	}
	
	public function build($runData){
		
		$pl = $runData->getParameterList();
		$lang = $pl->getParameterValue("lang");
		
		if($lang && $lang !== "pl" && $lang !== "en"){
			$lang = null;
		}
		
		$c = new Criteria();
		/*
		$c->add("flag_new_site", false);
		$c->add("page.site_id", 1, '!=');
		$c->addJoin("page_id", "page.page_id");
		$c->addOrderDescending("page_revision.revision_id");
		$c->setLimit(30);

		$revs = DB_PageRevisionPeer::instance()->select($c);
		
		// check for duplications
		$revs2 = array();
		
		foreach($revs as $r){
			$pageId = $r->getPageId();
			if($revs2[$pageId] == null){
				$revs2[$pageId] = $r;	
			}	
		}
		
		$revs2 = array_slice($revs2, 0, 10);
		
		$runData->contextAdd("revisions", $revs2);
			
		*/

		$q = "SELECT page.* FROM page, page_revision, site WHERE " .
				"page_revision.flag_new_site = FALSE ".
				"AND site.visible = TRUE AND site.private = FALSE 
				AND site.deleted = FALSE " ;
				
		if($lang){
			$q.= "AND site.language = '".db_escape_string($lang)."' ";	
		}
				
		$q.=		"AND page.site_id != 1".
				"AND page.revision_id = page_revision.revision_id ".
				"AND page.site_id = site.site_id " .
				"ORDER BY page.revision_id DESC LIMIT 10";
		$c->setExplicitQuery($q);
		
		$pages = DB_PagePeer::instance()->select($c);
		$runData->contextAdd("pages", $pages);
	}
	
}
