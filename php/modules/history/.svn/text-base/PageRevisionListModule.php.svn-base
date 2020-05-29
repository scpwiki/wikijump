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

class PageRevisionListModule extends SmartyModule {
	
	public function render($runData){
		$site = $runData->getTemp("site");
		
		$pl = $runData->getParameterList();
		$pageId = $pl->getParameterValue("page_id");
		$parmHash = md5(serialize($pl->asArray()));
		
		$key = 'pagehistory_v..'.$pageId.'..'.$parmHash;
		$tkey = 'pagerevisions_lc..'.$pageId; // last change timestamp

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

		$mc->set($key, $struct, 0, 3600);
		if(!$changeTimestamp){
			$changeTimestamp = $now;
			$mc->set($tkey, $changeTimestamp, 0, 3600);
		}

		return $out; 
	}
	
	public function build($runData){
		
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		
		$pageId = $pl->getParameterValue("page_id");
		$pageNumber = $pl->getParameterValue("page");
		$perPage = $pl->getParameterValue("perpage");
		
		$json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
		$o = $json->decode($pl->getParameterValue("options"));
		
		$offset = ($pageNumber - 1)*$perPage;
		$count = $perPage*2 + 1;
		
		// get the page
		
		$page = DB_PagePeer::instance()->selectByPrimaryKey($pageId);
		if($page == null || $page->getSiteId() !== $site->getSiteId()){
			throw new ProcessException(_("Error selecting the page."), "no_page");	
		}

		$c = new Criteria();
		$c->add('page_id', $pageId);
		
		// check options
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
		
		$c->addJoin("user_id", "ozone_user.user_id");
		
		$c->addOrderDescending('revision_id');
		$c->setLimit($count, $offset);
		
		$pr = DB_PageRevisionPeer::instance()->select($c);
		
		// now see if number of selected is equal $perPage + 1. If so - 
		// there is at least 1 more page to show...
		$counted = count($pr);
		$pagerData = array();
		$pagerData['current_page'] = $pageNumber;
		
		if($counted >$perPage*2){
				$knownPages=$pageNumber + 2;
				$pagerData['known_pages'] = $knownPages;	
		} elseif($counted >$perPage){
			$knownPages=$pageNumber + 1;
			$pagerData['total_pages'] = $knownPages;	
		} else {
			$totalPages = $pageNumber;	
			$pagerData['total_pages'] = $totalPages;
		}
		
		$pr = array_slice($pr, 0, $perPage);
		
		$runData->contextAdd("pagerData", $pagerData);
		
		$runData->contextAdd("page", $page);
		$runData->contextAdd("currentRevision", $page->getCurrentRevision());
		
		$runData->contextAdd("revisions", $pr);
	}
	
}
