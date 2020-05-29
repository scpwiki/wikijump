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

class SiteChangesListModule extends SmartyModule {
	
	public function render($runData){
		$site = $runData->getTemp("site");
		
		$pl = $runData->getParameterList();
		$parmHash = md5(serialize($pl->asArray()));
		
		$key = 'siterecentrevisions_v..'.$site->getUnixName().'..'.$parmHash;
		$tkey = 'siterevisions_lc..'.$site->getSiteId(); // last change timestamp

		$mc = OZONE::$memcache;
		$struct = $mc->get($key);
		$changeTimestamp = $mc->get($tkey);
	
		if($struct){
			// check the times
			$cacheTimestamp = $struct['timestamp'];
			
			// afford 1 minute delay
			if($changeTimestamp && $changeTimestamp <= $cacheTimestamp){
				return $struct['content'];	
			}
		}
		
		$out = parent::render($runData);
		
		// and store the data now
		$struct = array();
		$now = time();
		$struct['timestamp'] = $now;
		$struct['content'] = $out;

		$mc->set($key, $struct, 0, 600);
		if(!$changeTimestamp){
			$changeTimestamp = $now;
			$mc->set($tkey, $changeTimestamp, 0, 3600);
		}

		return $out; 
	}
	
	public function build($runData){
		// select recent revisions...
		
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
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
		$c->add("page_revision.site_id", $site->getSiteId());
		
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
		
		$categoryId = $pl->getParameterValue("categoryId");
		
		if($categoryId && is_numeric($categoryId)){
			$c->add("page.category_id", $categoryId);	
		}
		
		$c->addJoin("page_id", "page.page_id");
		$c->addJoin("user_id", "ozone_user.user_id");
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
