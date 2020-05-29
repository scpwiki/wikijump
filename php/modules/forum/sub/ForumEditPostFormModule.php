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

class ForumEditPostFormModule extends SmartyModule {
	
	public function build($runData){
		$pl = $runData->getParameterList();
		$postId = $pl->getParameterValue("postId", "AMODULE");
		$user = $runData->getUser();
		$site = $runData->getTemp("site");
		
		if($postId == null || !is_numeric($postId)){
			throw new ProcessException(_("No post specified."), "no_post");	
		}
		
		$post = DB_ForumPostPeer::instance()->selectByPrimaryKey($postId);
		if($post == null || $post->getSiteId() != $site->getSiteId()){
			throw new ProcessException(_("No post specified."), "no_post");	
		}	
		
		$category = $post->getForumThread()->getCategory();
		WDPermissionManager::instance()->hasForumPermission('edit_post', $runData->getUser(), $category, null, $post);	
		
		// check if thread blocked
		$thread = $post->getForumThread();
		if($thread->getBlocked()){
			// check if moderator or admin
			$c = new Criteria();
			$c->add("site_id", $site->getSiteId());
			$c->add("user_id", $user->getUserId());
			$rel = DB_ModeratorPeer::instance()->selectOne($c);
			if(!$rel || strpos($rel->getPermissions(), 'f') == false){
				$rel = DB_AdminPeer::instance()->selectOne($c);
				if(!$rel){
					throw new WDPermissionException(_("Sorry, this thread is blocked. Nobody can add new posts nor edit existing ones."));
				}
			}	
		}	
		
		// OK for now...
		
		// keep the session - i.e. put an object into session storage not to delete it!!!
		$runData->sessionAdd("keep", true);
		
		$runData->contextAdd("post", $post);
		
		$runData->ajaxResponseAdd("postId", $postId);
		
		$userId = $runData->getUserId();
		if($userId == null){
			$userString = $runData->createIpString();	
			$runData->contextAdd("anonymousString", $userString);
		}
		
	}
	
}
