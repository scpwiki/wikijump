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

class ForumPreviewPostModule extends SmartyModule {
	
	public function build($runData){
		$pl = $runData->getParameterList();
		$title = $pl->getParameterValue("title");
		$description = trim($pl->getParameterValue("description"));
		$source = trim($pl->getParameterValue("source"));
		
		if($source == null || $source == ''){
			throw new ProcessException(_("Post is empty."), "post_empty");	
		}
		
		$wt = new WikiTransformation();
		$wt->setMode('post');
		$body = $wt->processSource($source);
		
		$post = new DB_ForumPost();

		$post->setText($body);
		$post->setTitle($title);
		$post->setDatePosted(new ODate());
		
		// now set user_id, user_string
		
		$userId = $runData->getUserId();
		if($userId == null){
			$userString = $runData->createIpString();	
		}
		
		if($userId){
			$post->setUserId($userId);
		}else{
			$post->setUserId(0);
			$post->setUserString($userString);
		}
		
		$runData->contextAdd("post", $post);

	}
	
}
