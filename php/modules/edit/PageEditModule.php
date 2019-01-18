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

class PageEditModule extends SmartyModule {
	
	protected static $AUTOINCREMENT_PAGE = 'autoincrementpage';
	
	public function build($runData){
		
		$pl = $runData->getParameterList();
		$site = $runData->getTemp("site");
		
		$pageId = $pl->getParameterValue("page_id");
		
		$mode = $pl->getParameterValue("mode");
		
		$runData->ajaxResponseAdd("mode", $mode);
		
		$user = $runData->getUser();
		
		$userId = $runData->getUserId();
		if($userId == null){
			$userString = $runData->createIpString();	
			$runData->contextAdd("anonymousString", $userString);
		}
		
		$db = Database::connection();
		$db->begin();
		
		if($pageId === null || $pageId===''){
			// means probably creating a new page
			// no context is needed
			$runData->sessionStart();
			$mode = "page";
			$runData->contextAdd("mode", $mode);
			$runData->contextAdd("newPage", true);

			// first create if a page not already exists!
			$unixName = $pl->getParameterValue("wiki_page");
			$unixName = WDStringUtils::toUnixName($unixName); // purify! (for sure)
			
			if(!$unixName){
				throw new ProcessException(_("The page can not be found or does not exist."), "no_page");	
			}
		
			$page = DB_PagePeer::instance()->selectByName($site->getSiteId(), $unixName);
			if($page != null){
				// page exists!!! error!
				throw new ProcessException(_("The page you want to create already exists. Please refresh the page in your browser to see it."));
			/*	$runData->ajaxResponseAdd("pageExists", true);
				$runData->ajaxResponseAdd("locked", true); //well, it is somehow locked...
				$runData->setModuleTemplate("edit/NewPageExistsWinModule");
				$db->commit();
				return;	*/
			}
			
			// extract category name
			if(strpos($unixName, ':') != false){
				// ok, there is category!	
				$exp = explode(':',$unixName);
				$categoryName = $exp[0];
				$suggestedTitle = ucwords(str_replace("-", " ", $exp[1]));
			} else {
				// no category name, "_default" assumed
				$categoryName = "_default";	
				$suggestedTitle = ucwords(str_replace("-", " ", $unixName));
			}
			
			$stitle = $pl->getParameterValue("title");
			if($stitle){
				$suggestedTitle = $stitle;	
			}
			
			$category = DB_CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId());
			
			if($category == null){
				// get the default!
				//$category = DB_CategoryPeer::instance()->selectByName('_default', $site->getSiteId());
				$category = $this->createTempCategory($categoryName, $site);
			}
			
			// now check for permissions!!!
			WDPermissionManager::instance()->hasPagePermission('create', $user, $category);
			$autoincrement = false;
			if(preg_match(';^([a-z0-9]+:)?'.self::$AUTOINCREMENT_PAGE.'$;', $unixName)){
				$autoincrement = true;
			}
			if(!$autoincrement){
				$lock = new DB_PageEditLock();
				$lock->setPageUnixName($unixName);
				$lock->setSiteId($site->getSiteId());
				$lock->setUserId($runData->getUserId());
				$lock->setUserString($runData->getSession()->getIpAddress());
				
				$lock->setDateStarted(new ODate());
				$lock->setDateLastAccessed(new ODate());
				$lock->setMode("page");
				
				if($pl->getParameterValue("force_lock") != null){
					$lock->deleteConflicts();
				}else{	
					// check for conflicts
					$conflicts = $lock->getConflicts();
					if($conflicts != null){
						$runData->ajaxResponseAdd("locked", true); 
						$runData->setModuleTemplate("edit/NewPageLockedWinModule");	
						$runData->contextAdd("locks", $conflicts);
						return;
					}
				}
			
				$secret = md5(time().rand(1000,9999));
				$lock->setSecret($secret);
				$lock->setSessionId($runData->getSession()->getSessionId());
				$lock->save();
				$runData->ajaxResponseAdd('lock_id', $lock->getLockId());
				$runData->ajaxResponseAdd('lock_secret', $secret);
			} else {
				$runData->contextAdd('disableLocks', true);
				$runData->ajaxResponseAdd('disableLocks', true);
			}
			$runData->contextAdd("title", $suggestedTitle);
			
			
			/* Select available templates, but only if the category does not have a live template. */
			
            $templatePage = $category->getTemplatePage();
    		
            if ($templatePage && $form = Wikidot_Form::fromSource($templatePage->getSource())) {

                $runData->contextAdd("form", new Wikidot_Form_Renderer($form));

            } elseif (!$templatePage || !preg_match(';^={4,}$;sm', $templatePage->getSource())) {
        	    			
    			$templatesCategory = DB_CategoryPeer::instance()->selectByName("template", $site->getSiteId());
    		
    			if($templatesCategory != null){
    				$c = new Criteria();
    				$c->add("category_id", $templatesCategory->getCategoryId());
    				$c->addOrderAscending("title");
    				$templates =  DB_PagePeer::instance()->select($c);
    				
    				$runData->contextAdd("templates", $templates);
    				
    			}
    			
    			// check if there is a default template...
    
    			
    			if($category != null){
    				if($category->getTemplateId() != null){
    					$runData->contextAdd("templateId", $category->getTemplateId());
    					
    				}	
    			}
    		} else {
    		    /* Has default template, try to populate the edit box with initial content. */
    		    $templateSource = $templatePage->getSource();
        		$split = preg_split(';^={4,}$;sm', $templateSource);
    		    if(count($split) >= 2){
    	            /* Fine, there is some initial content. */
    	            $templateSource = trim(preg_replace(";^.*?\n={4,};s", '', $templateSource));   
    	        } else {
    	            $templateSource = '';
    	        }
    	        $runData->contextAdd('source', $templateSource);
    		}
			
			
			$db->commit();
			return;	
		}	
		
		// now if editing an existing page...
		
		if(!$pageId || !is_numeric($pageId)){
			throw new ProcessException(_("The page can not be found or does not exist."), "no_page");	
		}
		
		$page = DB_PagePeer::instance()->selectByPrimaryKey($pageId);
		if(!$page || $page->getSiteId() !== $site->getSiteId()){
			throw new ProcessException(_("The page can not be found or does not exist."), "no_page");	
		}
		
		$category = $page->getCategory();
		if($category == null){
			throw new ProcessException(_("Internal error - page category does not exist!!!"));	
		}
		
		// now check for permissions!
		
		WDPermissionManager::instance()->hasPagePermission('edit', $user, $category, $page);

        // now check if form is defined

        $templatePage = $category->getTemplatePage();

        if (preg_match('/^[^:]*:[^_]|^[^_:][^:]*$/', $page->getUnixName())
            && $templatePage && $form = Wikidot_Form::fromSource($templatePage->getSource())
        ) {
            $form->setDataFromYaml($page->getSource());
            $runData->contextAdd("form", new Wikidot_Form_Renderer($form));
		
		// check if mode is sections if page is editable in this mode
		} elseif($mode == "section"){
			$compiledContent = $page->getCompiled()->getText();
			$editable = WDEditUtils::sectionsEditable($compiledContent);
			if($editable == false){
				throw new ProcessException(_("Sections are not editable due to unclear section structure. This sometimes happen when nested headings are used (inside other page elements) or the page include other pages."), "sections_uneditable");
			}	
			// ok, get ranges for edit now.
			$pageSource = $page->getSource();
			$rangeMap = WDEditUtils::sectionMapping($pageSource);
			$sectionId = $pl->getParameterValue("section");

			if(!isset($rangeMap[$sectionId])){
				throw new ProcessException(_("Sections are not editable due to unclear section structure. This sometimes happen when nested headings are used (inside other page elements) or the page include other pages."), "sections_uneditable");
			}
			$rangeStart = $rangeMap[$sectionId]['start'];
			$rangeEnd = $rangeMap[$sectionId]['end'];
			
			$runData->ajaxResponseAdd('section', $sectionId);
			$runData->ajaxResponseAdd('rangeStart', $rangeStart);
			$runData->ajaxResponseAdd('rangeEnd', $rangeEnd);
			
		}

		// if we have not returned yet it means that the lock does not exist or is expired
		// if session is not started - start it!
		$runData->sessionStart();
		// create new page lock
		$lock = new DB_PageEditLock();
		$lock->setPageId($page->getPageId());
		$lock->setPageUnixName($page->getUnixName());
		$lock->setSiteId($site->getSiteId());
		$lock->setUserId($runData->getUserId());
		$lock->setUserString($runData->getSession()->getIpAddress());
		
		$lock->setDateStarted(new ODate());
		$lock->setDateLastAccessed(new ODate());
		$lock->setMode($mode);
		if($mode == "section"){
			$lock->setRangeStart($rangeStart);
			$lock->setRangeEnd($rangeEnd);	
		}
		
		// delete outdated...
		DB_PageEditLockPeer::instance()->deleteOutdated($pageId);
		// check for conflicts
		
		if($pl->getParameterValue("force_lock") != null){
			$lock->deleteConflicts();	
		} else {

			$blocklocks = $lock->getConflicts();
			if($blocklocks != null){
				// conflicting locks exist.	
				$runData->setModuleTemplate("edit/LockExistsWinModule");
				$runData->ajaxResponseAdd("locked", true);
				$runData->contextAdd("locks", $blocklocks);
				return;
			}
		}

		$secret = md5(time().rand(1000,9999));
		$lock->setSecret($secret);
		$lock->setSessionId($runData->getSession()->getSessionId());
		$lock->save();
		
		$runData->ajaxResponseAdd('lock_id', $lock->getLockId());
		$runData->ajaxResponseAdd('lock_secret', $secret);
		// also put current page revision in case one wants to regain lock after expired.
		
		$runData->ajaxResponseAdd('page_revision_id', $page->getRevisionId());
		
		// keep the session - i.e. put an object into session storage not to delete it!!!
		$runData->sessionAdd("keep", true);
		
		if($mode == "page"){
			$pageSource = $page->getSource();
			$runData->contextAdd("source", $pageSource);
		}
		if($mode == "append"){
			$runData->contextAdd("source", ""); // source not required...
		}
		if($mode == "section"){
			// slice the source...
			$sliced = explode("\n",$pageSource);
			$s = array_slice($sliced, $rangeStart, $rangeEnd-$rangeStart+1);
			
			$runData->contextAdd("source", trim(implode("\n", $s)));
		}
		$runData->contextAdd("title", $page->getTitleRaw());	
		$runData->contextAdd("pageId", $page->getPageId());	
		
		$runData->contextAdd("mode", $mode);
		
		$runData->ajaxResponseAdd("timeLeft", 15*60);
		
		$db->commit();
		
	}
	
	protected function createTempCategory($categoryName, $site){
		$category = DB_CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId(), false);
		if($category == null){
			// create the category - just clone the default category!!!
			$category = DB_CategoryPeer::instance()->selectByName("_default", $site->getSiteId(), false); 
			$category->setName($categoryName);
			// fill with some important things - we assume the _default category exists!!! IT REALLY SHOULD!!!
			$category->setCategoryId(null);
			$category->setNew(true); // this will make it INSERT, not UPDATE on save()
			$category->setPerPageDiscussion(null); //default value
			// set default permissions theme and license
			$category->setPermissionsDefault(true);
			$category->setThemeDefault(true);
			$category->setLicenseDefault(true);
			$category->setNavDefault(true);
		}
		
		return $category;
	}

}
