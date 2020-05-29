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

class SitesListByTagModule extends CacheableModule {
	
	protected $timeOut=300;
	
	public function build($runData){
		$pl = $runData->getParameterList();
		
		$tag = $pl->getParameterValue("tag");
		if($tag === null){
			$runData->setModuleTemplate("Empty");
			return ;	
		}	
		
		$lang = $pl->getParameterValue("lang");
		
		if($lang && $lang !== "pl" && $lang !== "en"){
			$lang = null;
		}

		// get sites
		
		$title = $pl->getParameterValue("title");
		$runData->contextAdd("title",$title);
		
		$c = new Criteria();
		$c->setExplicitFrom("site, site_tag");
		$c->add("site_tag.tag", $tag);
		$c->add("site.visible", true);
		$c->add("site.private", false);
		$c->add("site.deleted", false);
		if($lang){
			$c->add("site.language", $lang);	
		}
		$c->add("site_tag.site_id", "site.site_id", "=", false);
		$c->addOrderAscending('site.name');
		
		$sites = DB_SitePeer::instance()->select($c);
		
	//	$q = "SELECT site.* FROM site, tag WHERE tag.tag = '".db_escape_string($tag")."'
		
		$runData->contextAdd("tag", $tag);
		$runData->contextAdd("sites", $sites);
		$runData->contextAdd("sitesCount", count($sites));
	}
	
}
