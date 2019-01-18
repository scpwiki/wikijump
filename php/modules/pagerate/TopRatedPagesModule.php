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

class TopRatedPagesModule extends CacheableModule2 {
	
	protected $keyBase = 'top_rated_pages';
	protected $timeOut = 120;
	protected $delay = 0;
	
	public function build($runData){
		
		$site = $runData->getTemp("site");
		
		$pl = $runData->getParameterList();
		$limit =  $pl->getParameterValue("limit", "MODULE");
		
		if($limit === null|| !is_numeric($limit) || $limit<1 || $limit>300){
			$limit = 10;	
		}
		
		$order =$pl->getParameterValue("order");
		
		$minRating =$pl->getParameterValue("minRating");
		
		if($minRating !== null && !is_numeric($minRating)){
			$minRating = null;	
		}
		
		$maxRating =$pl->getParameterValue("maxRating");
		
		if($maxRating !== null && !is_numeric($maxRating)){
			$maxRating = null;	
		}
		
		$showComments = $pl->getParameterValue("comments", "MODULE");
		
		$categoryName = $pl->getParameterValue("category", "MODULE", "AMODULE");
		if($categoryName !== null){
			$category = DB_CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId());
			if($category == null){
				throw new ProcessException(_("The category can not be found."));	
			}
		}
		
		$c = new Criteria();
		if($category){
			$c->add("category_id", $category->getCategoryId());	
		}
		$c->add("site_id", $site->getSiteId());
		
		if($minRating!==null){
			$c->add("rate", $minRating,'>=');	
		}
		
		if($maxRating!==null){
			$c->add("rate", $maxRating,'<=');	
		}
		
		switch($order){
			
			case 'date-created-asc':
				$c->addOrderAscending("date_created");
				break;
			case 'date-created-desc':
				$c->addOrderDescending("date_created");
				break;
			case 'rate-asc':
				$c->addOrderAscending("rate");
				break;
			case 'rating-asc':
				$c->addOrderAscending("rate");
				break;
			default:
				$c->addOrderDescending("rate");
				break;
		}

		$c->addOrderAscending("COALESCE(title, unix_name)");
		if($limit){
			$c->setLimit($limit);
		}
		
		$pages = DB_PagePeer::instance()->select($c);
		
		if($showComments){
			
			foreach($pages as &$page){
				if($page->getThreadId()){
					$thread = DB_ForumThreadPeer::instance()->selectByPrimaryKey($page->getThreadId());
					$noc = $thread->getNumberPosts();	
				}else{
					$noc = 0;
				}
				$page->setTemp("numberComments", $noc);	
			}
			
		}
		
		$runData->contextAdd("pages", $pages);
			
	}
	
}
