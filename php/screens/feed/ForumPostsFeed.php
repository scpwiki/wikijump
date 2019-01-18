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

class ForumPostsFeed extends FeedScreen {
	
	public function render($runData){
		$site = $runData->getTemp("site");
		
		$pl = $runData->getParameterList();
		$parmHash = md5(serialize($pl->asArray()));
		
		$key = 'forumposts_f..'.$site->getUnixName().'..'.$parmHash;
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
			$mc->set($tkey, $changeTimestamp, 0, 1000);
		}
		if(!$allForumTimestamp){
			$allForumTimestamp = $now;
			$mc->set($akey, $allForumTimestamp, 0, 1000);
		}
			
		return $out; 
	}
	
	public function build($runData){
		$site = $runData->getTemp("site");
		
		$pl = $runData->getParameterList();
		$categoryId = $pl->getParameterValue("c");
		
		$channel = array();
		
		$channel['title'] = $site->getName()." - "._("new forum posts");
		$channel['link'] = "http://".$site->getDomain()."/forum/start";
		$channel['description'] = _("Posts in forums of the site"). " \"".$site->getName()."\"";
		if($site->getSubtitle()){
			$channel['description'] .=  " - ".$site->getSubtitle();
		}

		$items = array();
		
		$c = new Criteria();
		
		$c->add("forum_post.site_id", $site->getSiteId());
		$c->add("forum_group.visible", true);
		$c->addJoin("thread_id", "forum_thread.thread_id");
		$c->addJoin("user_id", "ozone_user.user_id");
		$c->addJoin("forum_thread.category_id", "forum_category.category_id");
		$c->addJoin("forum_category.group_id", "forum_group.group_id");
		$c->addOrderDescending("post_id");
		$c->setLimit(20);
		$posts = DB_ForumPostPeer::instance()->select($c);
		
		foreach($posts as $post){
			$item = array();

			$thread = $post->getForumThread();
			
			$item['title'] = $post->getTitle();
			$item['link'] = "http://".$site->getDomain()."/forum/t-".$thread->getThreadId().'/'.$thread->getUnixifiedTitle().'#post-'.$post->getPostId();
			$item['guid'] = "http://".$site->getDomain()."/forum/t-".$thread->getThreadId().'#post-'.$post->getPostId();
			$item['date'] = date('r', $post->getDatePosted()->getTimestamp());
			// TODO: replace relative links with absolute links!
			$content =  $post->getText();
			
			$content = preg_replace(';(<.*?)(src|href)="/([^"]+)"([^>]*>);si', '\\1\\2="http://'.$site->getDomain().'/\\3"\\4', $content);
			$content = preg_replace(';<script\s+[^>]+>.*?</script>;is', '', $content);
			$content = preg_replace(';(<[^>]*\s+)on[a-z]+="[^"]+"([^>]*>);si', '\\1 \\2', $content);

			// add extra info.
			
			$content .= '<br/>';
			$fcategory = $thread->getForumCategory();
			$content .= _('Forum category').': <a href="http://'.$site->getDomain().'/forum/c-'.$thread->getCategoryId().'">'.htmlspecialchars($fcategory->getForumGroup()->getName().' / '.$fcategory->getName()).'</a><br/>';
			$content .= _('Forum thread').': <a href="http://'.$site->getDomain().'/forum/t-'.$thread->getThreadId().'/'.$thread->getUnixifiedTitle().'">'
				.htmlspecialchars($thread->getTitle()).'</a>';
			
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
