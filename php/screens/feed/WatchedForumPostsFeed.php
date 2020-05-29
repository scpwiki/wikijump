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

class WatchedForumPostsFeed extends FeedScreen {
	
	protected $requiresAuthentication = true;
	
	public function render($runData){
		$user = $runData->getTemp("user");
		$key = "watchedforum..".$user->getUserId();
		$mc = OZONE::$memcache;
		$out = $mc->get($key);
		if($out){
			return $out;	
		}
		$out = parent::render($runData);
		$mc->set($key, $out, 0, 600);
		return $out;
	}
	
	public function build($runData){
	
		$user = $runData->getTemp("user");
		$userId = $user->getUserId();

		// set language for the user
		$lang = $user->getLanguage();
		$runData->setLanguage($lang);
		$GLOBALS['lang'] = $lang;
		
		// and for gettext too:
		
		switch($lang){
			case 'pl':
				$glang="pl_PL";
				break;
			case 'en':
				$glang="en_US";
				break;
		}

		putenv("LANG=$glang"); 
		putenv("LANGUAGE=$glang"); 
		setlocale(LC_ALL, $glang.'.UTF-8');
		
		// now just get watched page changes for the user...

		$c = new Criteria();
		
		$c->addJoin("thread_id", "forum_thread.thread_id");
		$c->addJoin("thread_id", "watched_forum_thread.thread_id");
		$c->addJoin("user_id", "ozone_user.user_id");
		$c->add("watched_forum_thread.user_id", $user->getUserId());
		$c->addOrderDescending("post_id");
		$c->setLimit(30);
		
		$posts = DB_ForumPostPeer::instance()->select($c);
		
		$channel['title'] = _('Wikidot.com watched forum discussions for user').' "'.$user->getNickName().'"';
		$channel['link'] = "http://" . GlobalProperties::$URL_HOST . "/account:you/start/watched-forum";
		
		$items = array();
		
		foreach($posts as $post){
			$thread = $post->getForumThread();
			
			$site = $post->getSite();
			
			$item = array();
		
			$item['title'] = $post->getTitle() . ' ('._('on site').' "'.htmlspecialchars($site->getName()).'")';
			$item['link'] = "http://".$site->getDomain()."/forum/t-".$thread->getThreadId().'/'.$thread->getUnixifiedTitle().'#post-'.$post->getPostId();
			$item['guid'] = "http://".$site->getDomain()."/forum/t-".$thread->getThreadId().'#post-'.$post->getPostId();
			
			$item['date'] = date('r', $post->getDatePosted()->getTimestamp());
			
			$content =  $post->getText();
			
			$content = preg_replace(';(<.*?)(src|href)="/([^"]+)"([^>]*>);si', '\\1\\2="http://'.$site->getDomain().'/\\3"\\4', $content);
			$content = preg_replace(';<script\s+[^>]+>.*?</script>;is', '', $content);
			$content = preg_replace(';(<[^>]*\s+)on[a-z]+="[^"]+"([^>]*>);si', '\\1 \\2', $content);
			
			// add extra info.
			
			$content .= '<br/><hr/>';
			$content .= _('Site').': <a href="http://'.$site->getDomain().'">'.htmlspecialchars($site->getName()).'</a><br/>';
			$content .= _('Forum category').': <a href="http://'.$site->getDomain().'/forum/c-'.$thread->getCategoryId().'">'.htmlspecialchars($thread->getForumCategory()->getName()).'</a><br/>';
			$content .= _('Forum thread').': <a href="http://'.$site->getDomain().'/forum/t-'.$thread->getThreadId().'/'.$thread->getUnixifiedTitle().'">'
				.htmlspecialchars($thread->getTitle()).'</a><br/>';
			$content .= _('Author of the post').': '.WDRenderUtils::renderUser($post->getUserOrString()).'<br/>';
			
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
