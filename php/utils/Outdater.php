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

class Outdater {
	
	private static $instance;
	
	private $vars = array();
	
	private $recurrenceLevel = 0;
	
	public static function instance(){
		if(self::$instance == null){
			self::$instance = new Outdater();
		}
		return self::$instance;	
	}
	
	public function __construct($baseRecurrenceLevel=0){
		$this->recurrenceLevel = $baseRecurrenceLevel +1;
	}	
	
	public function pageEvent($eventType, $page, $parm2 = null){
		
		if($this->recurrenceLevel >5) return;
		
		switch($eventType){
			case 'new_page':
				$this->recompilePage($page);
				$this->fixInLinks($page);
				$this->fixOutLinks($page);
				$this->fixInclusions($page);
				$this->recompileInclusionDeps($page);
				$this->outdatePageCache($page);
				$this->handleNavigationElement($page);
				$this->indexPage($page);
				$this->handleTemplateChange($page);
				break;
			case 'source_changed':
				$this->recompilePage($page);
				$this->outdatePageCache($page);
				$this->fixOutLinks($page);
				$this->fixInclusions($page);
				$this->recompileInclusionDeps($page);
				$this->handleNavigationElement($page);
				$this->indexPage($page);
				$this->handleTemplateChange($page);
				break;
			case 'title_changed':
				$this->recompilePage($page);
				$this->outdatePageCache($page);
				$this->outdateDescendantsCache($page);
				$this->fixInLinks($page); // if dynamical link text = page title
				$this->outdatePageTagsCache($page);
				$this->indexPage($page);
				break;
			case 'rename':
				// $parm2 is the old name
				$this->recompilePage($page);
				$this->fixInLinks($page);
				$this->fixInLinks($parm2);
				$this->recompileInclusionDeps($page);
				$this->recompileInclusionDeps($parm2);
				$this->outdateDescendantsCache($page);
				$this->outdatePageCache($parm2);
				$this->outdatePageCache($page);
				$this->outdatePageTagsCache($page);
				$this->indexPage($page);
				$this->handleTemplateChange($page);
				$this->handleTemplateChange($parm2);
				break;
			case 'delete':
				// $page is not just an old unix name. the page itself should be already deleted.
				$this->fixInLinks($page);
				$this->recompileInclusionDeps($page);
				//$this->outdateDescendantsCache($page); // this is done in Deleter
				$this->outdatePageTagsCache($page);
				$this->outdatePageCache($page);
				$this->handleTemplateChange($page);
				break;
			case 'parent_changed':
				$this->outdatePageCache($page);
				$this->outdateDescendantsCache($page);
				break;
			case 'file_change':
				$this->recompilePage($page);
				$this->outdatePageCache($page);
				$this->recompileInclusionDeps($page);
				break;
			case 'tag_change':
				//$this->outdatePageCache($page);
				//$this->outdatePageTagsCache($page);
				//$this->indexPage($page);
				$this->recompilePage($page);
				$this->outdatePageCache($page);
				$this->fixOutLinks($page);
				$this->fixInclusions($page);
				$this->recompileInclusionDeps($page);
				$this->handleNavigationElement($page);
				$this->indexPage($page);
				$this->handleTemplateChange($page);
				break;
			case 'page_vote':
				$this->outdatePageCache($page);
				$this->outdateRatingStars($page);
				break;
		}
		
		// reset vars
		$this->vars = array();
	}
	
	public function forumEvent($eventType, $parm=null){
		switch($eventType){
			case 'post_save': 
				// $parm is the post object here
				$this->handleForumPostSave($parm);
				break;	
			case 'thread_save': 
				// $parm is the post object here
				$this->handleForumThreadSave($parm);
				break;	
			case 'outdate_forum':
				$this->handleWholeForumOutdate();
				break;
		}	
	}
	
	public function categoryEvent($eventType, $category =null){
		switch($eventType){
			case 'category_save':
				$this->outdateCategoryPagesCache($category);
				break;
		}
	}
	
	public function themeEvent($eventType, $theme =null){
		
		switch($eventType){
			case 'theme_save':
				$this->outdateThemeDependentCategories($theme);
				break;
		}
	}

	public function siteEvent($eventType, $site =null){
		switch($eventType){
			case 'sitewide_change':
				$this->outdateAllPagesCache($site);
				break;
			case 'delete':
				$this->handleSiteDelete($site);
				break;
		}
	}
	
	/**
	 * This is the place where pages are compiled!
	 *
	 * @param Db_Page $page
	 */
	private function recompilePage($page){
		// compiled content not up to date. recompile!
		$source = $page->getSource(); 
			
		$c = new Criteria();
		$c->add("page_id", $page->getPageId());
		$compiled = DB_PageCompiledPeer::instance()->selectOne($c);

		/* Find out if the category is using any templates. */
		if(!preg_match(';(:|^)_;', $page->getUnixName())) {
    		$category = $page->getCategory();
    		$categoryName = $category->getName();
    	    $templatePage = DB_PagePeer::instance()->selectByName($page->getSiteId(), 
    		    ($categoryName == '_default' ? '' : $categoryName.':') .'_template');
    		
    		if($templatePage) {
        	    $source = $this->assemblySource($source, $templatePage->getSource(), $page);
    		}
		}
		$wt = new WikiTransformation();
		$wt->setPage($page);
		$result = $wt->processSource($source);
		
		$compiled->setText($result);
		$compiled->setDateCompiled(new ODate());
		$compiled->save();
		
		$linksExist = $wt->wiki->vars['internalLinksExist'];
		$linksNotExist = $wt->wiki->vars['internalLinksNotExist'];
		$inclusions = $wt->wiki->vars['inclusions'];
		$inclusionsNotExist = $wt->wiki->vars['inclusionsNotExist'];
		$externalLinks = $wt->wiki->vars['externalLinks'];

		$this->vars['linksExist'] = $linksExist;
		$this->vars['linksNotExist'] = $linksNotExist;
		$this->vars['inclusions'] = $inclusions;
		$this->vars['inclusionsNotExist'] = $inclusionsNotExist;
		$this->vars['externalLinks'] = $externalLinks;
		
	}
	
	private function assemblySource($source, $templateSource, $page = null){
	    $t = new WikiTransformation(false);
	    return $t->assemblyTemplate($source, $templateSource, $page);
	}
	
	/**
	 * Recompile pages that point to this page (named or unnamed links.
	 */
	private function fixInLinks($page){
		
		$site = $GLOBALS['site'];
		$c = new Criteria();
		$c->add("site_id",$site->getSiteId());
		if(is_string($page)){
			$c->add("to_page_name", $page);	
		}else{
			$c2=new Criteria();
			$c2->add("to_page_id", $page->getPageId());
			$c2->addOr("to_page_name", $page->getUnixName());
			$c->addCriteriaAnd($c2);
		}
		
		$dblinks = DB_PageLinkPeer::instance()->select($c);
		foreach($dblinks as $link){
			// get page
			$page = DB_PagePeer::instance()->selectByPrimaryKey($link->getFromPageId());
			$outdater = new Outdater($this->recurrenceLevel);
			$outdater->pageEvent("source_changed", $page);	
		}
	}
	
	/**
	 * Updates the table of links that originate from this page.
	 */
	private function fixOutLinks($page){
		
		$linksExist = $this->vars['linksExist'];
		$linksNotExist = $this->vars['linksNotExist'];
		// get links from the database first
		$c = new Criteria();
		$c->add("site_id", $page->getSiteId());
		$c->add("from_page_id", $page->getPageId());
		$c->add("to_page_name", null);
		$dblinks = DB_PageLinkPeer::instance()->select($c);
		// delete links from database that are not current
		if($linksExist == null && count($dblinks)>0){
			//delete all
			DB_PageLinkPeer::instance()->delete($c);
		}else{
			foreach($dblinks as $dblink){
				if($linksExist[$dblink->getToPageId()] == null){
					DB_PageLinkPeer::instance()->deleteByPrimaryKey($dblink->getLinkId());
				}else{
					// already in the database = remove from links to add
					unset($linksExist[$dblink->getToPageId()]);	
				}
			}	
		}	
		
		if($linksExist && count($linksExist)>0){
			// insert into database links that are not there yet.
			foreach ($linksExist as $link){
				$dblink = new DB_PageLink();
				$dblink->setFromPageId($page->getPageId());
				$dblink->setToPageId($link);
				$dblink->setSiteId($page->getSiteId());
				$dblink->save();	
			}
		}
		
		// NAMED LINKS
		
		// get links from the database first
		$c = new Criteria();
		$c->add("from_page_id", $page->getPageId());
		$c->add("to_page_id", null);
		$dblinks = DB_PageLinkPeer::instance()->select($c);
		
		// delete links from database that are not current
		if($linksNotExist == null && count($dblinks)>0){
			//delete all
			DB_PageLinkPeer::instance()->delete($c);
		}else{
			foreach($dblinks as $dblink){
				if($linksNotExist[$dblink->getToPageName()] == null){
					DB_PageLinkPeer::instance()->deleteByPrimaryKey($dblink->getLinkId());
				}else{
					// already in the database = remove from links to add
					unset($linksNotExist[$dblink->getToPageName()]);	
				}
			}
				
		}	
		
		if($linksNotExist && count($linksNotExist)>0){
			// insert into database links that are not there yet.
			foreach ($linksNotExist as $link){
				$dblink = new DB_PageLink();
				$dblink->setFromPageId($page->getPageId());
				$dblink->setToPageName($link);
				$dblink->setSiteId($page->getSiteId());
				$dblink->save();	
			}
		}
		
		/*
		 * Insert external links. 
		 */
		$externalLinks = $this->vars['externalLinks'];
		if(!$externalLinks){
			$externalLinks = array();
		}
		$externalLinks = $this->vars['externalLinks'];
		$c = new Criteria();
		$c->add("page_id", $page->getPageId());
		$dblinks = DB_PageExternalLinkPeer::instance()->select($c);
		
		/* From $externalLinks remove links that are already in $dblinks. */

		foreach($dblinks as $dblink){
			if(in_array($dblink->getToUrl(), $externalLinks)){
				unset($externalLinks[$dblink->getToUrl()]);
			} else {
				/* remove from database */
				DB_PageExternalLinkPeer::instance()->deleteByPrimaryKey($dblink->getLinkId());
			}
		}
		
		/* Now save new URLs. */
		$now = new ODate();
		if($externalLinks){
			foreach($externalLinks as $elink){
				$dblink = new DB_PageExternalLink();
				$dblink->setPageId($page->getPageId());
				$dblink->setSiteId($page->getSiteId());
				$dblink->setToUrl($elink);
				$dblink->setDate($now);
				$dblink->save();
			}
		}
	}
	
	/**
	 * Update table of inclusions - pages that are included by this page.
	 */
	private function fixInclusions($page){
		$inclusions = $this->vars['inclusions'];
		$c = new Criteria();
		$c->add("site_id", $page->getSiteId());
		$c->add("including_page_id", $page->getPageId());
		$c->add("included_page_name", null);
		
		$dbinclusions = DB_PageInclusionPeer::instance()->select($c);
		
		// delete inclusions from database that are not current
		if($inclusions == null && count($dbinclusions)>0){
			//delete all
			DB_PageInclusionPeer::instance()->delete($c);
		}else{
			foreach($dbinclusions as $dbinclusion){
				if($inclusions[$dbinclusion->getIncludedPageId()] == null){
					DB_PageLinkPeer::instance()->deleteByPrimaryKey($dbinclusion->getInclusionId());
				}else{
					// already in the database = remove from links to add
					unset($inclusions[$dbinclusion->getIncludedPageId()]);	
				}
			}
				
		}	
		
		if($inclusions && count($inclusions)>0){
			// insert into database links that are not there yet.
			foreach ($inclusions as $inclusion){
				$dbinclusion = new DB_PageInclusion();
				$dbinclusion->setIncludingPageId($page->getPageId());
				$dbinclusion->setIncludedPageId($inclusion);
				$dbinclusion->setSiteId($page->getSiteId());
				$dbinclusion->save();	
			}
		}

		// NAMED inclusions (where pages do not exist)
		
		// get links from the database first
		$c = new Criteria();
		$c->add("site_id", $page->getSiteId());
		$c->add("including_page_id", $page->getPageId());
		$c->add("included_page_id", null);
		$dblinks = DB_PageInclusionPeer::instance()->select($c);
		
		$linksNotExist = $this->vars['inclusionsNotExist'];
		
		// delete links from database that are not current
		if($linksNotExist == null && count($dblinks)>0){
			//delete all
			DB_PageInclusionPeer::instance()->delete($c);
		}else{
			foreach($dblinks as $dblink){
				if($linksNotExist[$dblink->getIncludedPageName()] == null){
					DB_PageInclusionPeer::instance()->deleteByPrimaryKey($dblink->getInclusionId());
				}else{
					// already in the database = remove from links to add
					unset($linksNotExist[$dblink->getIncludedPageName()]);	
				}
			}
				
		}	
		
		if($linksNotExist && count($linksNotExist)>0){
			// insert into database links that are not there yet.
			foreach ($linksNotExist as $link){
				$dblink = new DB_PageInclusion();
				$dblink->setIncludingPageId($page->getPageId());
				$dblink->setIncludedPageName($link);
				$dblink->setSiteId($page->getSiteId());
				$dblink->save();	
			}
		}
		
	}
	
	private function recompileInclusionDeps($page){
		// get deps
		$site = $GLOBALS['site'];
		$c = new Criteria();
		$c->add("site_id",$site->getSiteId());
		
		if(is_string($page)){
			$c->add("included_page_name", $page);	
		}else{
			$c2=new Criteria();
			$c2->add("included_page_id", $page->getPageId());
			$c2->addOr("included_page_name", $page->getUnixName());
			$c->addCriteriaAnd($c2);
		}

		$dbinclusions = DB_PageInclusionPeer::instance()->select($c);
		
		foreach($dbinclusions as $inc){
			$page = DB_PagePeer::instance()->selectByPrimaryKey($inc->getIncludingPageId());
			// triger source update (recompile)
			$outdater = new Outdater($this->recurrenceLevel);
			$outdater->pageEvent("source_changed", $page);	
		}
		
	}
	
	public function outdateDescendantsCache($page){
		// to keep breadcrumbs up-to-date
		
		//get all descendants.
		$rec = 0;
		
		$c = new Criteria();
		$c->add("parent_page_id", $page->getPageId());
		
		$pages = DB_PagePeer::instance()->select($c);
		
		while($pages !== null && count($pages)>0 && $rec<10){
			$p2 = array();
			foreach($pages as $p){
				$this->outdatePageCache($p);
				$c = new Criteria();
				$c->add("parent_page_id", $p->getPageId());
				$ptmp = DB_PagePeer::instance()->select($c);
				$p2 = array_merge($p2, $ptmp);
			}
			$pages = $p2;
			$rec++;
		}
		
	}
	
	public function outdatePageCache($page){
		// both levels!	
		$memcache = Ozone::$memcache;
		$site = $GLOBALS['site'];
		$now = time();
		if(is_string($page)){
			$pageName = $page;
		}else{
			$pageName = $page->getUnixName();
		}
		
		$key = 'url..'.$site->getUnixName() . '.' . GlobalProperties::$URL_DOMAIN . '/'.$pageName;
		$cd = $site->getCustomDomain();
		if($cd !== null && $cd !==''){
			$key = 'url..'.$cd.'/'.$pageName;
			$memcache->delete($key);
		}
		
		// check if default landing page
		if($site->getDefaultPage() == $pageName){
			$key = 'url..'.$site->getUnixName(). '.' . GlobalProperties::$URL_DOMAIN;
			if($cd !== null && $cd !==''){
				$key = 'url..'.$cd;
				$memcache->delete($key);
			}	
		}
		
		$memcache->delete($key);
		$key = 'page..'.$site->getUnixName().'..'.$pageName;
		$memcache->delete($key);
		
		/* Touch the catefory "last change" timestamp. */
		
	    if(strpos( $pageName, ":") != false){
			$tmp0 = explode(':',$pageName); 
			$categoryName = $tmp0[0];
		} else {
			$categoryName = "_default";
		}
		$ckey = 'pagecategory_lc..'.$site->getUnixName().'..'.$categoryName;
		$memcache->set($ckey, $now, 0, 10000);
		
		$ckey = 'pageall_lc..'.$site->getUnixName();
		$memcache->set($ckey, $now, 0, 10000);
		
		/*
		 * Outdate code blocks. 
		 */
		
		$ckey = 'pagecodeblocks..' . $site->getSiteId() . '..' . $pageName;
		$memcache->delete($ckey);
		
	}
	
	/**
	 * Check if this page is a navigation page for any of the categories.
	 * If so - clear cache of all pages in the category.
	 */
	private function handleNavigationElement($page){
		// get default cat
		$site = $GLOBALS['site'];
		$pUnixName = $page->getUnixName();
		$dcat = DB_CategoryPeer::instance()->selectByName('_default', $site->getSiteId());
		
		$q = "SELECT unix_name FROM page WHERE category_id IN ( " .
				"SELECT category_id FROM category WHERE nav_default = false " .
					"AND (top_bar_page_name='$pUnixName' OR side_bar_page_name='$pUnixName') " .
					"AND site_id='".$site->getSiteId()."'";
		if($dcat->getTopBarPageName() === $pUnixName || $dcat->getSideBarPageName() === $pUnixName){
			$q .= "UNION SELECT category_id FROM category WHERE nav_default = true " .
					"AND site_id='".$site->getSiteId()."'"; 
		}
		$q .= ")";
		
		$db = Database::connection();
		$r = $db->query($q);
		while($row = $r->nextRow()){
			$name = $row['unix_name'];
			$this->outdatePageCache($name);
		}
	}
	
	private function outdateCategoryPagesCache($category, $site = null){
		if(!$site){
			$site = DB_SitePeer::instance()->selectByPrimaryKey($category->getSiteId());	
		}
		
		$q = "SELECT unix_name FROM page WHERE category_id='".$category->getCategoryId()."'";
		$db = Database::connection();
		$r = $db->query($q);
		while($row = $r->nextRow()){
			$name = $row['unix_name'];
		}	
		
		// the above is not necesarily necessary. try the below code:
		
		$aKey = 'category_lc..'.$site->getUnixName().'..'.$category->getName();
		$mc = OZONE::$memcache;
		$now = time();
		$mc->set($aKey, $now, 0, 7200);
		$key = 'category..'.$site->getSiteId().'..'.$category->getName();
		$mc->delete($key);
		$key = 'categorybyid..'.$site->getSiteId().'..'.$category->getCategoryId();
		$mc->delete($key);
	}
	
	private function outdateThemeDependentCategories($theme){
		
		$c = new Criteria();
		$c->add("theme_id", $theme->getThemeId());
		$cats = DB_CategoryPeer::instance()->select($c);
		foreach($cats as $cat){
			$this->outdateCategoryPagesCache($cat);
		}	
	}
	
	private function outdateAllPagesCache($site){
		if(!$site){
			$site = $GLOBALS['site'];
		}
		$q = "SELECT unix_name FROM page WHERE site_id='".$site->getSiteId()."'";
		
		$db = Database::connection();
		$r = $db->query($q);
		while($row = $r->nextRow()){
			$name = $row['unix_name'];
			$this->outdatePageCache($name);
		}	
		// again the above is not necesarily necessary. try the below code:	
		$q = "SELECT name FROM category WHERE site_id='".$site->getSiteId()."'";
		
		$db = Database::connection();
		$r = $db->query($q);
		$mc = OZONE::$memcache;
		$now = time();
		while($row = $r->nextRow()){
			$name = $row['name'];
			$aKey = 'category_lc..'.$site->getUnixName().'..'.$name;
			$mc->set($aKey, $now, 0, 7200);
		}	
	}
	
	public function handleForumPostSave($post){
		$mc = Ozone::$memcache;
		// create an antry with mod time
		$now = time();
		$site = $GLOBALS['site'];
		
		// outdate forum thread
		$tkey = 'forumthread_lc..'.$site->getUnixName().'..'.$post->getThreadId();
		$mc->set($tkey, $now, 0, 1000);
		
		// outdate forum category
		$thread = $post->getForumThread();
		$tkey = 'forumcategory_lc..'.$site->getUnixName().'..'.$thread->getCategoryId();
		$mc->set($tkey, $now, 0, 1000);
		
		// outdate whole forum ;-)    (affects the main view)
		
		$tkey = 'forumstart_lc..'.$site->getUnixName();
		$mc->set($tkey, $now, 0, 1000);
		
		// check if forum not related to any page (page discussion)
		if($thread->getPageId() !== null){
			$page = DB_PagePeer::instance()->selectByPrimaryKey($thread->getPageId());
			$this->outdatePageCache($page);	
		}
		
	}
	
	public function handleForumThreadSave($thread){
		$mc = Ozone::$memcache;
		// create an antry with mod time
		$now = time();
		$site = $GLOBALS['site'];	
		
		// outdate forum thread
		$tkey = 'forumthread_lc..'.$site->getUnixName().'..'.$thread->getThreadId();
		$mc->set($tkey, $now, 0, 1000);
		
		// outdate forum category
		$tkey = 'forumcategory_lc..'.$site->getUnixName().'..'.$thread->getCategoryId();
		$mc->set($tkey, $now, 0, 1000);
		
		// outdate whole forum ;-)    (affects the main view)
		$tkey = 'forumstart_lc..'.$site->getUnixName();
		$mc->set($tkey, $now, 0, 1000);
		
	}
	
	private function handleWholeForumOutdate(){
		$mc = Ozone::$memcache;
		// create an antry with mod time
		$now = time();
		$site = $GLOBALS['site'];	
		
		$key = 'forumall_lc..'.$site->getUnixName();
		$mc->set($key, $now, 0, 3600);
	}
	
	public function recompileCategory($category){
		$site = DB_SitePeer::instance()->selectByPrimaryKey($category->getSiteId());
		$site0 = $GLOBALS['site'];
		$GLOBALS['site'] = $site;
		$c = new Criteria();
		$c->add("category_id", $category->getCategoryId());
		$pages = DB_PagePeer::instance()->select($c);
		
		foreach($pages as $page){
			$this->recompilePage($page);
			$this->outdatePageCache($page);
			$this->fixOutLinks($page);
			$this->fixInclusions($page);	
		}
		
		$GLOBALS['site']=$site0;
	}
	
	public function recompileWholeSite($site){
		$site0 = $GLOBALS['site'];
		$GLOBALS['site'] = $site;
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$pages = DB_PagePeer::instance()->select($c);
		
		foreach($pages as $page){
			$this->recompilePage($page);
			$this->outdatePageCache($page);
			$this->fixOutLinks($page);
			$this->fixInclusions($page);
			Indexer::instance()->indexPage($page);	
		}
		
		$GLOBALS['site']=$site0;
	}
	
	public function indexPage($page){
		Indexer::instance()->indexPage($page);
	}
	
	public function outdatePageTagsCache($page){
		
		if(is_string($page)){
			return;
		}else{
			$siteId = $page->getSiteId();
		}
		
		$key = "page_tags_lc..".$siteId;
		$mc = OZONE::$memcache;

		$mc->set($key, time(), 0, 3600);
			
	}
	
	public function outdateRatingStars($page){
		$siteId = $page->getSiteId();
		
		$key = "top_rated_pages_lc..".$siteId;
		$mc = OZONE::$memcache;

		$mc->set($key, time(), 0, 3600);
	}
	
	private function handleSiteDelete($site){
		$mc = OZONE::$memcache;
		
		$key = "sitesettings..".$site->getSiteId();
		$mc->delete($key);
		
		$key = 'site..'.$site->getUnixName();
		$mc->delete($key);
		
		$key = 'site_cd..'.$site->getCustomDomain();
		$mc->delete($key);	
	}
	
	private function handleCategoryDelete($category, $site = null){
		if(!$site){
			$site = DB_SitePeer::instance()->selectByPrimaryKey($category->getSiteId());	
		}	
		if(is_string($category)){
			$cname = $category->getName();	
		}else{
			$cname = $category;	
		}
		$key = 'category_lc..'.$site->getUnixName().'..'.$cname;
		$mc = OZONE::$memcache;
		$now = time();
		$mc->delete($key);
		$key = 'category..'.$site->getSiteId().'..'.$cname;
		$mc->delete($key);
		$key = 'categorybyid..'.$site->getSiteId().'..'.$cname;
		$mc->delete($key);
		
	}
	
	private function handleTemplateChange($page){
	    if(is_string($page)){
    	    if(strpos( $page, ":") != false){
    			$tmp0 = explode(':',$page); 
    			$categoryName = $tmp0[0];
    		} else {
    			$categoryName = "_default";
    		}
    		if(preg_match(';_template$;', $page)) {
    		    $site = $GLOBALS['site'];
    		    $category = DB_CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId(), false);
    		    $this->recompileCategory($category);
    		}
	    } elseif(preg_match(';_template$;', $page->getUnixName())) {
	        $category = $page->getCategory();
	        $this->recompileCategory($category);
        }
	}
	
}
