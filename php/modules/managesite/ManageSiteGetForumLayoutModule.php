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

class ManageSiteGetForumLayoutModule extends ManageSiteBaseModule {
	
	public function build($runData){
		
		// get all groups and categories, prepare them in a suitable form 
		$site = $runData->getTemp("site");
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->addOrderAscending("sort_index");

		$groups = DB_ForumGroupPeer::instance()->select($c);
		
		$g0 = array();
		$c0 = array();
		$gcount = 0;
		foreach($groups as $group){
			$grow = array();
			$grow['name']=$group->getName();
			$grow['description']=$group->getDescription();
			$grow['group_id']=$group->getGroupId();
			$grow['visible']=$group->getVisible();
	
			$g0[$gcount] = $grow;
			
			// now get categories...
			$c0[$gcount] = array();
			$c = new Criteria();
			$c->add("site_id", $site->getSiteId());
			$c->addOrderAscending("sort_index");
			$c->add("group_id", $group->getGroupId());
			$categories = DB_ForumCategoryPeer::instance()->select($c);
			$ccount = 0;
			foreach ($categories as $cat){
				$crow = array();
				$crow['name'] = $cat->getName();
				$crow['description'] = $cat->getDescription();
				$crow['category_id'] = $cat->getCategoryId();
				$crow['posts'] = $cat->getNumberPosts();
				$crow['number_threads'] = $cat->getNumberThreads();
				$crow['permissions'] = $cat->getPermissions();
				$crow['max_nest_level'] = $cat->getMaxNestLevel();
				
				$c0[$gcount][$ccount] = $crow;
				$ccount++;
			}
			
			$gcount++;
		}
		
		$runData->ajaxResponseAdd("groups", $g0);
		$runData->ajaxResponseAdd("categories", $c0);
		
		//get default nesting
		$fs = $site->getForumSettings();
		$runData->ajaxResponseAdd("defaultNesting", $fs->getMaxNestLevel());
			
	}
	
}
