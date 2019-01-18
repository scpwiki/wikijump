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

class UserChangesListModule extends SmartyLocalizedModule {
	
	public function build($runData){
		// select recent revisions...

		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		
		$userId = $pl->getParameterValue("userId");
		
		if($runData->getUser() && $userId == $runData->getUser()->getUserId()){
			$own = true;	
		}
		
		// get user
		
		//if($userId 
		
		$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($userId);
		
		if($user == null){
			throw new ProcessException(_("Error selecting user."), "no_user");	
		}
		
		// get options
		$pageNumber = $pl->getParameterValue("page");
		$op = $pl->getParameterValue("options");
		
		if($pageNumber === null){
			$pageNumber = 1;
		}
		
		if($op){
			$json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
			$o = $json->decode($op);
		}
		if(count($o) == 0){
			$o['all'] == true;	
		}
		
		$perPage = $pl->getParameterValue("perpage");
		if($perPage == null){$perPage=20;}
		
		$offset = ($pageNumber - 1)*$perPage;
		$count = $perPage*2 + 1;
		
		$c = new Criteria();
		
		$c->add("page_revision.user_id", $user->getUserId());
		if(!$own){
			$c->add("site.private", false);	
		}
		
		if(!$o['all'] && count($o)>0){
			$c2 = new Criteria();
			if($o['new']){ $c2->addOr("flag_new", true);}	
			if($o['source']){ $c2->addOr("flag_text", true);}	
			if($o['title']){ $c2->addOr("flag_title", true);}	
			if($o['move']){ $c2->addOr("flag_rename", true);}
			if($o['meta']){ $c2->addOr("flag_meta", true);}
			if($o['files']){ $c2->addOr("flag_file", true);}
			$c->addCriteriaAnd($c2);		
		}

		$c->addJoin("page_revision.page_id", "page.page_id");
		$c->addJoin("page.site_id", "site.site_id");
		$c->add("site.deleted", false);
		$c->addOrderDescending("page_revision.revision_id");
		$c->setLimit($count, $offset);
		$revisions = DB_PageRevisionPeer::instance()->select($c);
		
		$counted = count($revisions);
		$pagerData = array();
		$pagerData['currentPage'] = $pageNumber;
		if($counted >$perPage*2){
			$knownPages=$pageNumber + 2;
			$pagerData['knownPages'] = $knownPages;	
		}elseif($counted >$perPage){
			$knownPages=$pageNumber + 1;
			$pagerData['totalPages'] = $knownPages;	
		} else {
			$totalPages = $pageNumber;	
			$pagerData['totalPages'] = $totalPages;
		}
		
		$revisions = array_slice($revisions, 0, $perPage);
		$runData->contextAdd("pagerData", $pagerData);
		$runData->contextAdd("revisions", $revisions);
		$runData->contextAdd("revisionsCount", count($revisions));
		
	}
	
}
