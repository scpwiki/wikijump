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

class ForumPostRevisionsModule extends SmartyModule {
	
	public function build($runData){
		$pl = $runData->getParameterList();
		$postId = $pl->getParameterValue("postId");
		
		$site = $runData->getTemp("site");
		
		if($postId == null || !is_numeric($postId)){
			throw new ProcessException(_("No post specified."), "no_post");	
		}
		
		$post = DB_ForumPostPeer::instance()->selectByPrimaryKey($postId);
		if($post == null || $post->getSiteId() != $site->getSiteId()){
			throw new ProcessException(_("No post specified."), "no_post");	
		}	
		
		// get all revisions
		
		$c = new Criteria();
		$c->add("post_id", $postId);
		$c->addOrderDescending("revision_id");
		
		$revs = DB_ForumPostRevisionPeer::instance()->select($c);
		
		$runData->contextAdd("revisions", $revs);
		$runData->contextAdd("post", $post);
		
		$runData->ajaxResponseAdd("postId", $postId);
	}
	
}
