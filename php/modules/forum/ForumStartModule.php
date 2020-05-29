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

class ForumStartModule extends SmartyModule {
	
	protected $processPage = true;
	
	public function render($runData){
		$site = $runData->getTemp("site");
		
		$pl = $runData->getParameterList();
		$parmHash = md5(serialize($pl->asArray()));
		
		$key = 'forumstart_v..'.$site->getUnixName().'..'.$parmHash;
		$tkey = 'forumstart_lc..'.$site->getUnixName(); // last change timestamp
		$akey = 'forumall_lc..'.$site->getUnixName();
		
		$mc = OZONE::$memcache;
		$struct = $mc->get($key);
		$cacheTimestamp = $struct['timestamp'];
		$changeTimestamp = $mc->get($tkey);
		$allForumTimestamp = $mc->get($akey);
		if($struct){
			// check the times
			
			if($changeTimestamp && $changeTimestamp <= $cacheTimestamp && $allForumTimestamp && $allForumTimestamp <= $cacheTimestamp){
				return $struct['content'];	
			}
		}
		
		$out = parent::render($runData);
		
		// and store the data now
		$struct = array();
		$now = time();
		$struct['timestamp'] = $now;
		$struct['content'] = $out;

		$mc->set($key, $struct, 0, 864000);
		
		if(!$changeTimestamp){
			$changeTimestamp = $now;
			$mc->set($tkey, $changeTimestamp, 0, 864000);
		}
		if(!$allForumTimestamp){
			$allForumTimestamp = $now;
			$mc->set($akey, $allForumTimestamp, 0, 864000);
		}
			
		return $out; 
	}
	
	public function build($runData){
		
		$pl = $runData->getParameterList();
		
		$site = $runData->getTemp("site");
		// get groups and categories
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		if(!$pl->getParameterValue("hidden")){
			$c->add("visible", true);
			$runData->contextAdd("hidden", true);
		}
		$c->addOrderAscending("sort_index");
		
		$groups = DB_ForumGroupPeer::instance()->select($c);
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->addOrderAscending("sort_index");
		
		$categories = DB_ForumCategoryPeer::instance()->select($c);
		
		// now mangle the categories and put into array
		// - in order to avoid several queries
		
		$cats = array();
		foreach($categories as $category){
			$cats[$category->getGroupId()][] = $category;
		}
		
		$runData->contextAdd("groups", $groups);	
		$runData->contextAdd("catarray", $cats);
			
	}
	
	public function processPage($out, $runData){
		$site = $runData->getTemp("site");
		$link = '/feed/forum/threads.xml';
		$title =  $site->getName()." - "._("new forum threads");
		$out = preg_replace("/<\/head>/", 
				'<link rel="alternate" type="application/rss+xml" title="'.preg_quote_replacement(htmlspecialchars($title)).'" href="'.$link.'"/></head>',
				$out,1);
				
		$link = '/feed/forum/posts.xml';
		$title =  $site->getName()." - new forum posts";
		$out = preg_replace("/<\/head>/", 
				'<link rel="alternate" type="application/rss+xml" title="'.preg_quote_replacement(htmlspecialchars($title)).'" href="'.$link.'"/></head>',
				$out,1);
					
		return $out;
	}
}
