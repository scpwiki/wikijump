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

class ForumDeletePostModule extends SmartyModule {
	
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
		try{
			WDPermissionManager::instance()->hasForumPermission('moderate_forum', $runData->getUser(), $category);
		}catch(Exception $e){
			throw new WDPermissionException(_("Sorry, you are not allowed to delete posts. Only site administrators and moderators are the ones who can."));	
		}

		// OK for now...
		//check if there any child posts
		
		$c = new Criteria();
		$c->add("parent_id", $postId);
		$chpc =  DB_ForumPostPeer::instance()->selectCount($c);
		
		if($chpc>0){
			$runData->contextAdd("hasChildren", true);	
		}
		
		$runData->contextAdd("post", $post);
		
		$runData->ajaxResponseAdd("postId", $postId);
		
	}
	
}
