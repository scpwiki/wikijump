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

class ForumMiniRecentPostsModule extends CacheableModule {
	
	protected $timeOut = 300;
	
	public function build($runData){
		
		$site = $runData->getTemp("site");
		
		$pl = $runData->getParameterList();
		$limit =  $pl->getParameterValue("limit", "MODULE");
		
		if($limit == null|| !is_numeric($limit) || $limit<1 || $limit>300){
			$limit = 5;	
		}
		
		$categoryId = $pl->getParameterValue("categoryId", "MODULE", "AMODULE");
		if($categoryId !== null){
			$category = DB_ForumCategoryPeer::instance()->selectByPrimaryKey($categoryId);
			if($category == null || $category->getSiteId() != $site->getSiteId()){
				throw new ProcessException(_("The category can not be found."));	
			}
		}
		
		// get recent forum posts
		
		$c = new Criteria();
		$c->add("forum_post.site_id", $site->getSiteId());
		if($category){
			$c->add("forum_post.category_id", $category->getCategoryId());	
		}
		$c->addJoin("thread_id", "forum_thread.thread_id");
		$c->addOrderDescending("post_id");
		$c->setLimit($limit);
		
		$posts = DB_ForumPostPeer::instance()->select($c);

		$runData->contextAdd("posts", $posts);

	}
	
}
