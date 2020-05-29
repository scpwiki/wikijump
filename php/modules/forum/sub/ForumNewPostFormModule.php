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

class ForumNewPostFormModule extends SmartyModule {
	
	public function build($runData){

		$pl = $runData->getParameterList();
		$postId = $pl->getParameterValue("postId");
		$threadId = $pl->getParameterValue("threadId");
		
		$user = $runData->getUser();
		
		$site = $runData->getTemp("site");
		
		$title = '';
		
		$db = Database::connection();
		$db->begin();

		$thread = DB_ForumThreadPeer::instance()->selectByPrimaryKey($threadId);
		if($thread == null || $thread->getSiteId() !== $site->getSiteId()){
			throw new ProcessException(_("No thread found... Is it deleted?"), "no_thread");
		}
		
		// check if thread blocked
		if($thread->getBlocked()){
			// check if moderator or admin
			if($runData->getUser()){
				$c = new Criteria();
				$c->add("site_id", $site->getSiteId());
				$c->add("user_id", $user->getUserId());
				$rel = DB_ModeratorPeer::instance()->selectOne($c);
			}
			if(!$rel || strpos($rel->getPermissions(), 'f') == false){
				$rel = DB_AdminPeer::instance()->selectOne($c);
				if(!$rel){
					throw new WDPermissionException(_("Sorry, this thread is blocked. Nobody can add new posts nor edit existing ones."));
				}
			}	
		}

		// now check if user is allowed
		
		$category = $thread->getCategory();
		WDPermissionManager::instance()->hasForumPermission('new_post', $runData->getUser(), $category);

		if($postId !== null && is_numeric($postId)){
			$post = DB_ForumPostPeer::instance()->selectByPrimaryKey($postId);
			if($post == null || $post->getThreadId() !== $thread->getThreadId()){
				throw new ProcessException(_("Original post does not exist! Please reload the page to make it up-to-date."), "no_post");	
			}
			
			// try to  determine true parent id based on the nesting level.
			// TODO!
			$maxNest = $thread->getForumCategory()->getEffectiveMaxNestLevel();
			// now check the nest level of the post... woooo...
			
			$parentId6 = $post->getParentId();
			$nestLevel6 = 0;
			$parents = array();
			while($parentId6 != null){
				$parent6 = DB_ForumPostPeer::instance()->selectByPrimaryKey($parentId6);
				$parents[] = $parent6;
				$parentId6 = $parent6->getParentId();
				$nestLevel6++;	
			}
			if($nestLevel6>=$maxNest){
				// change parent id to the maxNest-1 in the chain	
				$parent = ($parents[$nestLevel6 - ($maxNest-1)-1]);
				if($parent) {
					$parentId = $parent->getPostId();
					$parentChanged = true;
				}
			}else{
				$parentId = $post->getPostId();
			}
				$title = preg_replace('/^Re:\s?/', '', $post->getTitle());
				$title = 'Re: '.$title;
		}else{
			// only if NOT a page discussion
			if($thread->getPageId() == null){
				$title = 'Re: '.$thread->getTitle();
			}	
		}

		$runData->contextAdd("thread", $thread);
		if($parentId){
			$runData->ajaxResponseAdd("parentId", $parentId);
			$runData->contextAdd("parentId", $parentId);
			if($parentChanged){
				$runData->ajaxResponseAdd("parentChanged", true);
				
			}	
		}
		
		$runData->contextAdd("title", $title);
		
		// keep the session - i.e. put an object into session storage not to delete it!!!
		$runData->sessionAdd("keep", true);
		
		$userId = $runData->getUserId();
		if($userId == null){
			$userString = $runData->createIpString();	
			$runData->contextAdd("anonymousString", $userString);
		}
		
		$db->commit();	
		
	}
	
}
