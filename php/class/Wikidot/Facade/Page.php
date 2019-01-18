<?php

class Wikidot_Facade_Page extends Wikidot_Facade_Base {
	/**
	 * Get all page attributes from site
	 * 
	 * Argument array keys:
	 *  site: site to get page from
	 *  page: page to get (full_name)
	 * 
	 * @param struct $args
	 * @return struct
	 */
	public function get($args) {
		$this->parseArgs($args, array("performer", "site", "page"));
		
		WDPermissionManager::instance()->canAccessSite($this->performer, $this->site);
		
		return $this->repr($this->page);
	}
	
	/**
	 * Get files from page
	 * 
	 * Argument array keys:
	 *  site: site to get page from
	 *  page: page to get (full_name) files from
	 * 
	 * @param struct $args
	 * @return struct
	 */
	public function files($args) {
		$this->parseArgs($args, array("performer", "site", "page"));
		
		WDPermissionManager::instance()->canAccessSite($this->performer, $this->site);
		
		$c = new Criteria();
		$c->add("page_id", $this->page->getPageId());
		$files = DB_FilePeer::instance()->select($c);
		
		return $this->repr($files);
    }

    private function _getOrCreateCategory($site, $categoryName) {
        $category = DB_CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId(), false);
        if ($category == null){
            // create the category - just clone the default category!!!
            $category = DB_CategoryPeer::instance()->selectByName("_default", $site->getSiteId(), false);
            $category->setCategoryId(null);
            $category->setNew(true);
            $category->setName($categoryName);
            // fill with some important things - we assume the _default category exists!!! IT REALLY SHOULD!!!
            $category->setPerPageDiscussion(null); //default value
            // set default permissions theme and license
            $category->setPermissionsDefault(true);
            $category->setThemeDefault(true);
            $category->setLicenseDefault(true);
            $category->setNavDefault(true);
            $category->save();
        }
        return $category;
     }
	
	public function save($args) {
		
		$db = Database::connection();
		$db->begin();
		
		// simple argument checking
		if (! isset($args['page'])) {
			throw new Wikidot_Facade_Exception_WrongArguments("Page argument must be passed");
		}
		
		$pm = new WDPermissionManager();
		$now = new ODate();
		
		// page (existant or not) name
		$arg_page = WDStringUtils::toUnixName($args['page']);
		
		// parse the rest (beside page name)
		unset($args['page']);
		$this->parseArgs($args, array("performer", "site"));
		
		try {
			
			// parse page name to figure out if it points to an existant page
			$page = $this->_parsePage($this->site, $arg_page);
			
			$new = false;

			// check permissions to edit the page
			$pm->hasPagePermission('edit', $this->performer, $page->getCategory(), $page);

		} catch (Wikidot_Facade_Exception_WrongArguments $e) {
            if ($this->source === null) {
                $this->source = "";
            }
            if ($this->title === null) {
                $this->title = $arg_page;
            }
            $new = true;

            $category_name = preg_replace('/^([^:]*):.*$/', '\1', $arg_page);
            if ($category_name == $arg_page) {
                $category_name = '_default';
            }
            $category = $this->_getOrCreateCategory($this->site, $category_name);

            $page = new DB_Page();
            $page->setSiteId($this->site->getSiteId());
            $page->setCategoryId($category->getCategoryId());
            $page->setUnixName($arg_page);
            $page->setDateCreated(new ODate());
            $page->setOwnerUserId($this->performer->getUserId());
            $page->save();

            $compiled = new DB_PageCompiled();
            $compiled->setPageId($page->getPageId());
            $compiled->save();
		}
		
    	// get current revision and metadata
        if (! $new) {
    		$cur_rev = $page->getCurrentRevision();
	        $cur_meta = $cur_rev->getMetadata();
        }
			
		// construct new metadata
        if ($new) {
            $new_meta = new DB_PageMetadata();
            $new_meta->setUnixName($arg_page);
            $new_meta->setOwnerUserId($this->performer->getUserId());
        } else {
    		$new_meta = clone $cur_meta;
	    	$new_meta->setNew(true);
    		$new_meta->setMetadataId(null);
        }
		
        // construct new revision
		$new_rev = new DB_PageRevision();
		$new_rev->setSiteId($this->site->getSiteId());
		$new_rev->setPageId($page->getPageId());
        $new_rev->setUserId($this->performer->getUserId());
		$new_rev->setDateLastEdited($now);
        if ($new) {
            $new_rev->setRevisionNumber(0);
        } else {
    		$new_rev->setRevisionNumber($cur_rev->getRevisionNumber() + 1);
        }
		
		$src_changed = false;
		$title_changed = false;
		$parent_changed = false;
		$tags_changed = false;
		
		// handle source change
		if ($new || ($this->source !== null && $page->getSource() != $this->source)) {
			
			$new_src = new DB_PageSource();
			$new_src->setText($this->source);
			$new_src->save();
			
			$new_rev->setSourceId($new_src->getSourceId());

			$src_changed = true;
		
		} else {
			
			$new_rev->setSourceId($cur_rev->getSourceId());
			$new_rev->setSinceFullSource($cur_rev->getSinceFullSource());
			$new_rev->setDiffSource($cur_rev->getDiffSource());
			
		}
		
		// handle tags change
		if ($this->tags) {
			
			$new_tags = $this->tags;
			$cur_tags = $page->getTagsAsArray();
			
			sort($cur_tags);
			sort($new_tags);
			
			if ($cur_tags != $new_tags) {
				$tags_changed = true;
				$tags_deleted = array();
				$tags_added = array();
				
				foreach ($cur_tags as $tag) {
					if (! in_array($tag, $new_tags)) {
						
						$c = new Criteria();
						$c->add('page_id', $page->getPageId());
						$c->add('tag', $tag);
						
						if ($t = DB_PageTagPeer::instance()->selectOne($c)) {
							$t->delete();
							$tags_deleted[] = $tag;
						}
					}
				}
				
				foreach ($new_tags as $tag) {
					if (! in_array($tag, $cur_tags)) {
						$t = new DB_PageTag();
						$t->getPageId($page->getPageId());
						$t->setSiteId($this->site->getSiteId());
						$t->setTag($tag);
						$t->save();
						
						$tags_added[] = $tag;
					}
				}
			}
		}
		
		// handle metadata: title change
		if ($new || ($this->title !== null && $cur_meta->getTitle() != $this->title)) {
			
			$new_meta->setTitle($this->title);
			$page->setTitle($this->title);
			$title_changed = true;
		}
		
		// handle metadata: parent page change
		if ($this->parent_page) {
			if (! $cur_meta->getParentPageId() || 
			    $cur_meta->getParentPageId() != $this->parent_page->getPageId()
			) {
				$new_meta->setParentPageId($this->parent_page->getPageId());
				$parent_changed = true;
			}
		}
		if ($this->clear_parent_page && $page->getParentPageId()) {
			$new_meta->setParentPageId(null);
			$parent_changed = true;
		}
		
		$meta_changed = $title_changed || $parent_changed;
		
		// decide whether to use previous metadata or create a new object
		if ($meta_changed) {
			
			$new_meta->save();
			$new_rev->setMetadataId($new_meta->getMetadataId());
			
		} else {
			$new_rev->setMetadataId($cur_meta->getMetadataId());
		}

        // set flag on revision
        if ($new) {
            $new_rev->setFlagNew(true);
        } else {
            if ($src_changed) {
                $new_rev->setFlagText(true);
            }
            if ($title_changed) {
                $new_rev->setFlagTitle(true);
            }
            if ($parent_changed) {
                $new_rev->setFlagMeta(true);
            }
        }
		
		if ($src_changed || $meta_changed || $tags_changed) {
			
			$new_rev->save();
			
			$page->setSourceId($new_rev->getSourceId());
			$page->setDateLastEdited($now);
			$page->setMetadataId($new_rev->getMetadataId());	
			$page->setRevisionNumber($new_rev->getRevisionNumber());
			$page->setRevisionId($new_rev->getRevisionId());
			$page->save();
			
			$db->commit();
		
            $GLOBALS['site'] = $this->site;
			$outdater = new Outdater();
			if ($src_changed) {
				$outdater->pageEvent("source_changed", $page);
			}
			if ($title_changed) {
				$outdater->pageEvent("title_changed", $page);
			}
			if ($parent_changed) {
				$outdater->pageEvent("parent_changed", $page);
			}
			if ($tags_changed) {
				$outdater->pageEvent("tag_changed", $page);
			}
			
		} else {
			
			/* This place is reached when API client tries to set source or
			 * title or parent page or tags that are already set (in the DB)
			 * to the same value.
			 * 
			 * Let's suppose doing nothing is the desired behavior in this case
			 * 
			 * Other possible way to react can be raising an exception.
			 * But it should be different from Wikidot_Facade_Exception_WrongArguments
			 * because this one implies client error (and client does not need
			 * to know the exact database state).
			 */
			
		}
	}
}
