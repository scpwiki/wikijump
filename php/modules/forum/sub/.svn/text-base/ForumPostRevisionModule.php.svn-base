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

class ForumPostRevisionModule extends SmartyModule {
	
	public function build($runData){
		$pl = $runData->getParameterList();
		$revisionId = $pl->getParameterValue("revisionId");
		
		$site = $runData->getTemp("site");
		
		if($revisionId == null || !is_numeric($revisionId)){
			throw new ProcessException(_("No revision specified."), "no_post");	
		}
		
		$revision = DB_ForumPostRevisionPeer::instance()->selectByPrimaryKey($revisionId);
		if($revision == null){
			throw new ProcessException(_("No revision specified."), "no_post");	
		}	
		
		$runData->ajaxResponseAdd("title", $revision->getTitle());
		
		$source = $revision->getText();
		$wt = new WikiTransformation();
		$body = $wt->processSource($source);
		
		$runData->ajaxResponseAdd("content", $body);
		$runData->ajaxResponseAdd("postId", $revision->getPostId());

	}
	
}
