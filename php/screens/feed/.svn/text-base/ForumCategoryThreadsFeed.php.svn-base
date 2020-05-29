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

class ForumCategoryThreadsFeed extends FeedScreen {
	
	public function render($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$categoryId = $pl->getParameterValue("c");
		
		$parmHash = md5(serialize($pl->asArray()));
		
		$key = 'forumcategorythreads_f..'.$site->getUnixName().'..'.$categoryId.'..'.$parmHash;
		$tkey = 'forumcategory_lc..'.$site->getUnixName().'..'.$categoryId; // last change timestamp
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
		
		$mc->set($key, $struct, 0, 1000);
		if(!$changeTimestamp){
			$changeTimestamp = $now;
			$mc->set($tkey, $changeTimestamp, 0, 1000);
		}
		if(!$allForumTimestamp){
			$allForumTimestamp = $now;
			$mc->set($akey, $allForumTimestamp, 0, 10000);
		}
			
		return $out; 
	}
	
	public function build($runData){
	
		$site = $runData->getTemp("site");
		
		$pl = $runData->getParameterList();
		$categoryId = $pl->getParameterValue("c");
		
		$category = DB_ForumCategoryPeer::instance()->selectByPrimaryKey($categoryId);
		if($category == null){
			throw new ProcessException(_("No such category."), "no_category");	
		}
		$channel = array();
		
		$channel['title'] = $category->getName()." (new threads)";
		$channel['link'] = "http://".$site->getDomain()."/forum/c-".$categoryId."/".$category->getUnixifiedName();
		$channel['description'] = _("Threads in the forum category")." \"".$category->getName()."\"";
		if($category->getDescription()){
			$channel['description'] .=  " - ".$category->getDescription();
		}

		$items = array();
		
		$c = new Criteria();
		$c->add("category_id", $categoryId);
		$c->addJoin("user_id", "ozone_user.user_id");
		$c->addOrderDescending("thread_id");
		$c->setLimit(20);
		$threads = DB_ForumThreadPeer::instance()->select($c);
		
		foreach($threads as $thread){
			$item = array();
			
			$item['title'] = $thread->getTitle();
			$item['link'] = "http://".$site->getDomain()."/forum/t-".$thread->getThreadId().'/'.$thread->getUnixifiedTitle();
			$item['guid'] = "http://".$site->getDomain()."/forum/t-".$thread->getThreadId();
			$item['date'] = date('r', $thread->getDateStarted()->getTimestamp());
			
			//replace relative links with absolute links!
			$post = $thread->getFirstPost();
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
