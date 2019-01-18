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

class ForumViewCategoryModule extends SmartyModule {
	
	private $category;
	protected $processPage = true;

	public function render($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$categoryId = $pl->getParameterValue("c");
		
		$parmHash = md5(serialize($pl->asArray()));
		
		$key = 'forumcategory_v..'.$site->getUnixName().'..'.$categoryId.'..'.$parmHash;
		$tkey = 'forumcategory_lc..'.$site->getUnixName().'..'.$categoryId; // last change timestamp
		$akey = 'forumall_lc..'.$site->getUnixName();
		
		$mc = OZONE::$memcache;
		$struct = $mc->get($key);
		$cacheTimestamp = $struct['timestamp'];
		$changeTimestamp = $mc->get($tkey);
		$allForumTimestamp = $mc->get($akey);
		if($struct){
			// check the times
			
			if($changeTimestamp && $changeTimestamp <= $cacheTimestamp && $allForumTimestamp && $allForumTimestamp <= $cacheTimestamp){
				$this->categoryName = $struct['categoryName'];
				$this->categoryId = $struct['categoryId'];
				return $struct['content'];	
			}
		}
		
		$out = parent::render($runData);
		
		// and store the data now
		$struct = array();
		$now = time();
		$struct['timestamp'] = $now;
		$struct['content'] = $out;
		$struct['categoryName']=$this->categoryName;
		$struct['categoryId']=$this->categoryId;

		$mc->set($key, $struct, 0, 864000);
		if(!$changeTimestamp){
			$changeTimestamp = $now;
			$mc->set($tkey, $changeTimestamp, 0, 864000);
		}
		if(!$allForumTimestamp){
			$allForumTimestamp = $now;
			$mc->set($akey, $allForumTimestamp, 0, 864000);
		}
			
		return $out; 
	}
	
	public function build($runData){
		
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$categoryId = $pl->getParameterValue("c");
		
		$pageNumber = $pl->getParameterValue("p");
		if($pageNumber == null || !is_numeric($pageNumber) || $pageNumber <1){
			$pageNumber = 1;	
		}
		
		if($categoryId === null || !is_numeric($categoryId)){
			throw new ProcessException(_("No forum category has been specified."), "no_category");	
		}
		
		$sort = $pl->getParameterValue("sort");
		
		$c = new Criteria();
		$c->add("category_id", $categoryId);
		$c->add("site_id", $site->getSiteId());
		
		$category = DB_ForumCategoryPeer::instance()->selectOne($c);
		
		if($category == null || $category->getSiteId() !== $site->getSiteId()){
			throw new ProcessException(_("Requested forum category does not exist."), "no_category");	
		}
		
		$this->categoryName = $category->getName();
		$this->categoryId = $category->getCategoryId();
		// select threads...
		
		$perPage = 20;
		$offset = ($pageNumber - 1)*$perPage;
		$pagerData = array();
		$pagerData['current_page'] = $pageNumber;
		$pagerData['total_pages'] = ceil($category->getNumberThreads() / $perPage);

		$c = new Criteria();
		$c->add("category_id", $categoryId);
		$c->add("site_id", $site->getSiteId());
		$c->addOrderDescending("sticky");
		
		if($sort == "start"){
			$c->addOrderDescending("thread_id");
		}else{
			//$c->addOrderDescending("last_post_id", "NULLS LAST"); // sorry, requires postgresql 8.3?
			$c->addOrderDescending('COALESCE(last_post_id, 0)');
			$c->addOrderDescending("thread_id");
		}
		$c->setLimit($perPage, $offset);
		
		$threads = DB_ForumThreadPeer::instance()->select($c);
		
		$runData->contextAdd("pagerData", $pagerData);
		$runData->contextAdd("category", $category);
		$runData->contextAdd("threads", $threads);
		$runData->contextAdd("threadsCount", count($threads));
		$runData->contextAdd("sortStart", $sort=="start");
	}
	
	public function processPage($out, $runData){
		if($this->categoryName != null){
			$pageTitle = $this->categoryName;
			$runData->getTemp("page")->setTitle($pageTitle); // DANGEROUS!!! DO NOT SAVE THE PAGE AFTER THIS!!!
			$out = preg_replace("/<title>(.+?)<\/title>/is","<title>\\1 ".preg_quote_replacement(htmlspecialchars($pageTitle))."</title>",$out,1);
			
			$out = preg_replace("/<div id=\"page-title\">(.*?)<\/div>/is","<div id=\"page-title\">".htmlspecialchars($this->categoryName)."</div>",$out, 1);
			
			// feeds!
			$link = '/feed/forum/cp-'.$this->categoryId.'.xml';
			$out = preg_replace("/<\/head>/", 
				'<link rel="alternate" type="application/rss+xml" title="'._('Posts in the forum category').' &quot;'.preg_quote_replacement(htmlspecialchars($this->categoryName)).'&quot;" href="'.$link.'"/></head>',
				$out,1);
			$link = '/feed/forum/ct-'.$this->categoryId.'.xml';
			$out = preg_replace("/<\/head>/", 
				'<link rel="alternate" type="application/rss+xml" title="'._('Threads in the forum category').' &quot;'.preg_quote_replacement(htmlspecialchars($this->categoryName)).'&quot;" href="'.$link.'"/></head>',
				$out,1);

		}
		return $out;	
	}
	
}
