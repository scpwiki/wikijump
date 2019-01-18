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

class UserRecentPostsListModule extends SmartyLocalizedModule {
	
	public function build($runData){
		
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		
		$userId = $pl->getParameterValue("userId");

		if($runData->getUser() && $userId == $runData->getUser()->getUserId()){
			$own = true;	
		}
		
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
		$c->add("forum_post.user_id", $userId);
		if(!$own){
			$c->add("site.private", false);	
		}
		$c->addJoin("thread_id", "forum_thread.thread_id");
		$c->addJoin("user_id", "ozone_user.user_id");
		$c->addJoin("forum_post.site_id", "site.site_id");
		$c->add("site.deleted", false);
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
