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

class ForumRecentPostsListModule extends SmartyModule {
	
	public function render($runData){
		$site = $runData->getTemp("site");
		
		$pl = $runData->getParameterList();
		$parmHash = md5(serialize($pl->asArray()));
		
		$key = 'forumrecentposts_v..'.$site->getUnixName().'..'.$parmHash;
		$tkey = 'forumstart_lc..'.$site->getUnixName(); // last change timestamp
		$akey = 'forumall_lc..'.$site->getUnixName();
		
		$mc = OZONE::$memcache;
		$struct = $mc->get($key);
		$changeTimestamp = $mc->get($tkey);
		$allForumTimestamp = $mc->get($akey);
		if($struct){
			// check the times
			$cacheTimestamp = $struct['timestamp'];
			
			// afford 1 minute delay
			if($changeTimestamp && $changeTimestamp <= $cacheTimestamp+60 && $allForumTimestamp && $allForumTimestamp <= $cacheTimestamp){
				return $struct['content'];	
			}
		}
		
		$out = parent::render($runData);
		
		// and store the data now
		$struct = array();
		$now = time();
		$struct['timestamp'] = $now;
		$struct['content'] = $out;

		$mc->set($key, $struct, 0, 1000);
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
		
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		
		$categoryId = $pl->getParameterValue("categoryId");
		$limit = $pl->getParameterValue("limit");
		
		if($limit == null || !is_numeric($limit)){
			$limit = 20;	
		}
		
		$pageNumber = $pl->getParameterValue("page");
		$op = $pl->getParameterValue("options");
		
		if($pageNumber === null){
			$pageNumber = 1;
		}
		$perPage = $limit;
		$offset = ($pageNumber - 1)*$perPage;
		$count = $perPage*2 + 1;
		
		$c = new Criteria();
		if($categoryId !== null && is_numeric($categoryId)){
			$c->add("forum_thread.category_id", $categoryId);	
		}
		$c->add("forum_post.site_id", $site->getSiteId());
		$c->addJoin("thread_id", "forum_thread.thread_id");
		$c->addJoin("user_id", "ozone_user.user_id");
		$c->addJoin("forum_thread.category_id", "forum_category.category_id");
		$c->addOrderDescending("post_id");
		$c->setLimit($count, $offset);
		$posts = DB_ForumPostPeer::instance()->select($c);
		
		$counted = count($posts);
		$pagerData = array();
		$pagerData['currentPage'] = $pageNumber;
		if($counted >$perPage*2){
			$knownPages=$pageNumber + 2;
			$pagerData['knownPages'] = $knownPages;	
		}elseif($counted >$perPage){
			$knownPages=$pageNumber + 1;
			$pagerData['totalPages'] = $knownPages;	
		} else {
			$totalPages = $pageNumber;	
			$pagerData['totalPages'] = $totalPages;
		}
		$posts = array_slice($posts, 0, $perPage);
		
		$runData->contextAdd("pagerData", $pagerData);
		
		$runData->contextAdd("posts", $posts);
			
	}
	
}
