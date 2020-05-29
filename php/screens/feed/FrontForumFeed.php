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

class FrontForumFeed extends FeedScreen {
	
	public function render($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		
		$pageName = $pl->getParameterValue("page");
		$label = $pl->getParameterValue("label");
		
		$key = 'frontforumfeed..'.$site->getUnixName().'..'.$pageName.'..'.$label;
		
		$valid = true;
		
		$mc = OZONE::$memcache;
		$struct = $mc->get($key);
		if(!$struct){
			$valid = false;
		}
		$cacheTimestamp = $struct['timestamp'];
		
		$fkey = "frontforumfeedobject..".$site->getUnixName().'..'.$pageName.'..'.$label;
		$feed = $mc->get($fkey);
		
		if(!$feed){	
			$page = DB_PagePeer::instance()->selectByName($site->getSiteId(), $pageName);
		
			// 	get the feed object	
			$c = new Criteria();
			$c->add("page_id", $page->getPageId());
			$c->add("label", $label);
		
			$feed = DB_FrontForumFeedPeer::instance()->selectOne($c);
			$mc->set($fkey, $feed, 0, 3600);
		}
		
		$now = time();
		
		$categoryIds = $feed->getCategories();
		
		// now check lc for ALL categories involved
		$cats = preg_split('/[,;] ?/', $categoryIds);
		
		foreach($cats as $cat){
		
			$tkey = 'forumcategory_lc..'.$site->getUnixName().'..'.$cat; // last change timestamp
			$changeTimestamp = $mc->get($tkey);
			if($changeTimestamp && $cacheTimestamp && $changeTimestamp <= $cacheTimestamp){
				//cache valid	
			}else{
				$valid = false;
				if(!$changeTimestamp){
					// 	put timestamp
					$mc->set($tkey, $now, 0, 10000);
					$valid = false;
				}	
			}
		}
		$akey = 'forumall_lc..'.$site->getUnixName();
		$allForumTimestamp = $mc->get($akey);
		if($allForumTimestamp &&  $cacheTimestamp && $changeTimestamp <= $cacheTimestamp){
			//cache valid
		}else{
			if(!$allForumTimestamp){
				$mc->set($akey, $now, 0, 10000);
			}	
		}
		
		if($valid){
			return $struct['content'];	
		}
		
		$out = parent::render($runData);
		
		// and store the data now
		$struct = array();
		$now = time();
		$struct['timestamp'] = $now;
		$struct['content'] = $out;

		$mc->set($key, $struct, 0, 1000);
		
		return $out; 
	}
	
	public function build($runData){
	
		$site = $runData->getTemp("site");
		
		$pl = $runData->getParameterList();
		
		$pageName = $pl->getParameterValue("page");
		$label = $pl->getParameterValue("label");
		
		// get the feed object
		$page = DB_PagePeer::instance()->selectByName($site->getSiteId(), $pageName);
		if(!$page){
			throw new ProcessException(_("No such page."), "no_page");	
		}
		$c = new Criteria();
		$c->add("page_id", $page->getPageId());
		$c->add("label", $label);
		$feed = DB_FrontForumFeedPeer::instance()->selectOne($c);

		$categoryIds = $feed->getCategories();
		$cats = preg_split('/[,;] ?/', $categoryIds);
		
		$ccat = new Criteria();
		$categories = array();
		
		if(count($cats)>20){
			throw new ProcessException(_("Maximum number of categories exceeded."), "max_categories");	
		}
		
		// get page
		$page = DB_PagePeer::instance()->selectByPrimaryKey($feed->getPageId());
		if(!$page){
			throw new ProcessException(_("Page can not be found."), "no_page");
		}
		
		foreach($cats as $categoryId){
		
			if($categoryId === null || !is_numeric($categoryId)){
				throw new ProcessException(_('Problem parsing attribute "category".'),"no_category");	
			}

			$category = DB_ForumCategoryPeer::instance()->selectByPrimaryKey($categoryId);
		
			if($category == null || $category->getSiteId() !== $site->getSiteId()){
				throw new ProcessException(_("Requested forum category does not exist."), "no_category");	
			}
			
			$categories[$category->getCategoryId()] = $category;
			$ccat->addOr("category_id", $category->getCategoryId());
		}
		$c = new Criteria();
		$c->addCriteriaAnd($ccat);

		$c->addOrderDescending("thread_id");
		$c->setLimit(30);
		$threads = DB_ForumThreadPeer::instance()->select($c);

		$channel['title'] = $feed->getTitle();
		$channel['link'] = "http://".$site->getDomain()."/".$page->getUnixName();
		if($feed->getDescription()){
			$channel['description'] = $feed->getDescription();
		}

		$items = array();

		foreach($threads as $thread){
			$item = array();
			
			$item['title'] = $thread->getTitle();
			$item['link'] = "http://".$site->getDomain()."/forum/t-".$thread->getThreadId().'/'.$thread->getUnixifiedTitle();
			$item['guid'] = $item['link'];
			$item['date'] = date('r', $thread->getDateStarted()->getTimestamp());
			
			$item['category'] = $thread->getCategory()->getName();
			
			//replace relative links with absolute links!
			$post = $thread->getFirstPost();
			if(!$post){
				continue;
			}
			
			$content =  $post->getText();
			
			$content = preg_replace(';(<.*?)(src|href)="/([^"]+)"([^>]*>);si', '\\1\\2="http://'.$site->getDomain().'/\\3"\\4', $content);
			$content = preg_replace(';<script\s+[^>]+>.*?</script>;is', '', $content);
			$content = preg_replace(';(<[^>]*\s+)on[a-z]+="[^"]+"([^>]*>);si', '\\1 \\2', $content);

			if($thread->getDescription()){
				$item['description'] = $thread->getDescription();	
			}
			
			$item['content'] = $content;
			if($post->getUserId()>0){
				$item['authorUserId'] = $post->getUserId();	
				$user = $post->getUser();
				$item['author']=$user->getNickName();
			}else{
				$item['author']=$post->getUserString();	
			}
			$items[] = $item;	
		}
		
		$runData->contextAdd("channel", $channel);
		$runData->contextAdd("items", $items);
		
	}
	
}
