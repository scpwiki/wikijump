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
 * @package Wikidot_Web
 * @version $Id: Lucene.php,v 1.1 2008/12/10 13:00:21 quake Exp $
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

class Wikidot_Search_Lucene {
	
	protected $AT_ONCE = 10;	// load N pages from DB at once when indexing
	protected $CACHE_FOR = 150;	// cache the results for seconds
	
	protected $indexFile;
	protected $index;
	protected $queueFile;
	protected $processedFtsEntries = array();
	
	public function __construct($indexFile = null, $queueFile = null) {
		if ($indexFile) {
			$this->indexFile = $indexFile;
		} else {
			$this->indexFile = GlobalProperties::$SEARCH_LUCENE_INDEX;
		}
		
		if ($queueFile) {
			$this->queueFile = $queueFile;
		} else {
			$this->queueFile = GlobalProperties::$SEARCH_LUCENE_QUEUE;
		}
	}
	
	protected function loadIndex() {
		$this->index = Zend_Search_Lucene::open($this->indexFile);
		Zend_Search_Lucene_Analysis_Analyzer::setDefault(new Zend_Search_Lucene_Analysis_Analyzer_Common_Utf8Num_CaseInsensitive());
	}
	
	public function createIndex() {
		$this->index = Zend_Search_Lucene::create($this->indexFile);
		Zend_Search_Lucene_Analysis_Analyzer::setDefault(new Zend_Search_Lucene_Analysis_Analyzer_Common_Utf8Num_CaseInsensitive());
	}
	
	protected function resetQueue() {
		file_put_contents($this->queueFile, "");
	}
	
	protected function queue($type, $id) {
		$fp = fopen($this->queueFile, "a");
		
		if (! in_array($type, array("INDEX_FTS", "INDEX_SITE", "DELETE_PAGE", "DELETE_THREAD", "DELETE_SITE"))) {
			$type = "UNKNOWN";
		}
		$id = (int) $id;
		
		fwrite($fp, "$type $id\n");
		fclose($fp);
	}
	
	protected function deleteItems($query) {
		foreach ($this->index->find($query) as $hit) {
			$this->index->delete($hit->id);
		}
		
		$this->index->commit();
	}
	
	protected function addFtsEntry($fts, $site = null) {
		if ($fts) {
			if (in_array($fts->getFtsId(), $this->processedFtsEntries)) {
				return;
			}
			
			$this->processedFtsEntries[] = $fts->getFtsId();
			
			if (! $site) {
				$site = DB_SitePeer::instance()->selectByPrimaryKey($fts->getSiteId());
			}
			
			if (! $site || $site->getDeleted() || ! $site->getVisible()) {
				return;
			}
			
			// delete it first
			$this->deleteItems("fts_id:" . $fts->getFtsId());
			
			// construct the document
			$doc = new Zend_Search_Lucene_Document();
			
			// add content, site_id, site_public, fts_id fields
			$doc->addField(Zend_Search_Lucene_Field::unStored("content", $fts->getText()));
			$doc->addField(Zend_Search_Lucene_Field::text("site_id", $fts->getSiteId()));
			$doc->addField(Zend_Search_Lucene_Field::text("site_public", $site->getPrivate() ? "false" : "true"));
			$doc->addField(Zend_Search_Lucene_Field::text("fts_id", $fts->getFtsId()));
			
			// TITLE
			$title_field = Zend_Search_Lucene_Field::text("title", $fts->getTitle());
			$title_field->boost = 7;
			$doc->addField($title_field);
			
			if ($fts->getPageId()) {
				
				// delete also by page_id (this shouldn't delete anything more)
				$this->deleteItems("page_id:" . $fts->getPageId());
				
				$doc->addField(Zend_Search_Lucene_Field::text("item_type", "page"));
				$doc->addField(Zend_Search_Lucene_Field::text("page_id", $fts->getPageId()));
				
				// TAGS
				if ($page = DB_PagePeer::instance()->selectByPrimaryKey($fts->getPageId())) {
					
					$tags = $page->getTagsAsArray();
					$tags_field = Zend_Search_Lucene_Field::text("tags", implode(" ", $tags));
					$tags_field->boost = 4 * count($tags);
					$doc->addField($tags_field);
					
				}
				
			} elseif ($fts->getThreadId()) {
				
				// delete also by thread_id (this shouldn't delete anything more)
				$this->deleteItems("thread_id:" . $fts->getThreadId());
				
				$doc->addField(Zend_Search_Lucene_Field::keyword("item_type", "thread"));
				$doc->addField(Zend_Search_Lucene_Field::keyword("thread_id", $fts->getThreadId()));
				
			} else {
				
				// NEITHER A PAGE NOR THREAD
				return;
				
			}
			
			$this->index->addDocument($doc);
		}
	}
	
	protected function indexSite($site, $verbose = false, $fts_id_from = null, $fts_id_to = null) {
		
		if ($site) {
		
			$atOnce = $this->AT_ONCE;
			$offset = 0;
			
			$c = new Criteria();
			$c->setLimit($atOnce, $offset);
			
			if ($fts_id_to) {
				$c->add("fts_id", $fts_id_from, ">=");
				$c->add("fts_id", $fts_id_to, "<");
			}
			
			if ($site == "ALL") {
				$site = null;
			} else {
				$c->add("site_id", $site->getSiteId());
			}
			
			$pp = DB_FtsEntryPeer::instance();
			$entries = null;
			
			do {
				unset($entries); // try to save SOME memory
				
				$entries = $pp->selectByCriteria($c);
				
				foreach ($entries as $fts) {
					$this->addFtsEntry($fts, $site);
				}

				$offset += $atOnce;
				$c->setLimit($atOnce, $offset);
				
				if ($verbose) {
					echo ".";
				}
				
			} while (count($entries));
		}
	}
	
	public function processQueue() {
		$this->loadIndex();
		
		$q = file($this->queueFile);
		$this->resetQueue();
		
		foreach ($q as $msg) {
			$m = explode(" ", $msg);
			$type = $m[0];
			$id = $m[1];
			
			if ($type == "INDEX_FTS") {
				
				$fts = DB_FtsEntryPeer::instance()->selectByPrimaryKey($id);
				$this->addFtsEntry($fts);
				
			} elseif ($type == "INDEX_SITE") {
				
				$this->indexSite(DB_SitePeer::instance()->selectByPrimaryKey($id));
				
			} elseif ($type == "DELETE_PAGE") {
				
				$this->deleteItems("page_id:$id");
				
			} elseif ($type == "DELETE_THREAD") {
				
				$this->deleteItems("thread_id:$id");
				
			} elseif ($type == "DELETE_SITE") {
				
				$this->deleteItems("site_id:$id");
				
			}
		}
	}
	
	public function queueFtsEntry($fts_id) {
		$this->queue("INDEX_FTS", $fts_id);
	}
	
	public function queueDeletePage($page_id) {
		$this->queue("DELETE_PAGE", $page_id);
	}
	
	public function queueDeleteThread($thread_id) {
		$this->queue("DELETE_THREAD", $thread_id);
	}
	
	public function queueReIndexSite($site_id) {
		$this->queue("DELETE_SITE", $site_id);
		$this->queue("INDEX_SITE", $site_id);
	}

	/*
	 * queries the index and returns the array of Fts entries
	 * @param $query Lucene query to search for
	 * @return array fts_id array
	 */
	public function rawQuery($query) {
		$cache = Ozone::$memcache;
		$key = "search.." . md5($query);
		
		if ($cache && $result = $cache->get($key)) {
			return $result;
		}
		
		$result = $this->executeWikidotSearch($query);
		
		if ($cache) {
			$cache->set($key, $result, 0, $this->CACHE_FOR);
		}
		
		return $result;
	}
	
	/**
	 * high level Wikidot search function
	 * manages user permisisons, searches only in public sites + those user is a member of
	 * 
	 * @param $phrase		 Lucene query to search for 
	 * @param $user			 user that searches
	 * @param $itemType		 p - search only pages, f - only forums
	 * @param $sites		 sites to search within
	 * @param $onlyUserSites whether to search ONLY in user sites 
	 * @return array  		 fts_id array
	 */
	public function search($phrase, $user = null, $itemType = null, $sites = null, $onlyUserSites = false) {
		
		// user filter
		
		if ($onlyUserSites) {
			$user_query = "";
		} else {
			$user_query = "site_public:true";
		}
		
		if ($user) {
			$c = new Criteria();
			$c->add("user_id", $user->getUserId());
			$c->setLimit(100, 0);
			
			$memberships = DB_MemberPeer::instance()->selectByCriteria($c);
			if (count($memberships) < 100) {
				foreach ($memberships as $m) {
					$user_query .= " site_id:" . $m->getSiteId() . "^2";
				}
			}
		}
		
		if ($user_query == "") {
			$user_query = "site_public:true";
		}
		
		// sites filter
		
		$sites_query = "";
		if (is_array($sites) && count($sites)) {
			foreach ($sites as $site) {
				if (! is_numeric($site)) { // not an ID
					if (is_string($site)) { // maybe unix_name?
						$c = new Criteria();
						$c->add("unix_name", $site);
						$site = DB_SitePeer::instance()->selectOne($c); // make it an object
					}
				}
				if (is_a($site, "DB_Site")) { // object?
					$site = $site->getSiteId(); // get an id
				}
				if ($site !== null && is_numeric($site)) { // we have site id finally
					$sites_query .= " site_id:$site";
				}
			}	
		}
		
		// construct content_query
		$phrase = trim($phrase);
		if ($phrase == "") {
			return array();
		}
		if (! preg_match("/tags:/", $phrase) && ! preg_match("/title:/", $phrase) && ! preg_match("/content:/", $phrase)) {
			
			// give the exact match in title higher boost
			if (! strstr($phrase, '"') && ! strstr($phrase, '^')) {
				$title_phrase = "\"$phrase\"^5 $phrase";
			} else {
				$title_phrase = $phrase;
			}
			
			$content_query = "tags:($phrase) title:($title_phrase) content:($phrase)";
		} else {
			$content_query = $phrase;
		}
	
		$query = "";
		if ($item_type == "p") {
			$query .= "+item_type:page ";
		}
		if ($item_type == "f") {
			$query .= "+item_type:thread ";
		}
		if ($sites_query) {
			$query .= "+($sites_query) ";
		}
		$query .= "+($user_query) +($content_query)";
		
		return $this->rawQuery($query);
	}
	
	public function indexAllSitesVerbose($fts_id_from = null, $fts_id_to = null) {
		$this->loadIndex();
		$this->indexSite("ALL", true, $fts_id_from, $fts_id_to);
		echo "\n";
	}
	
	protected function executeWikidotSearch($query) {
		$results = array();
		if (GlobalProperties::$SEARCH_USE_JAVA) {
			$cmd = "java -jar " . escapeshellcmd(WIKIDOT_ROOT . "/bin/wikidot_search.jar");
			$cmd .= " " . escapeshellarg($this->indexFile);
			$cmd .= " " . escapeshellarg($query);
			$cmd .= " 2>&1";
			
			exec($cmd, $results);
			if (count($results)) {
				// something other than int in the first line means we had an exception in java program
				if (! is_numeric($results[0])) {
					throw new Wikidot_Search_Exception(join("\n", $results));
				}
			}
		} else {
			$this->loadIndex();
			foreach ($this->index->find($query) as $hit) {
				$results[] = $hit->fts_id;
			}
		}
		
		return $results;
	}
	
	public function getCount() {
		$this->loadIndex();
		return $this->index->count();
	}
}
