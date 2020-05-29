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

class WatchAction extends SmartyAction {
	
	public function isAllowed($runData){
		$userId = $runData->getUserId();
		if($userId == null || $userId <1){
			throw new WDPermissionException(_("This option is available only to registered (and logged-in) users."));
		}
		return true;
	}
	
	public function perform($r){}
	
	public function watchThreadEvent($runData){
		$pl = $runData->getParameterList();
		
		$threadId = $pl->getParameterValue('threadId');
		if($threadId === null || !is_numeric($threadId)){
			throw new ProcessException(_("Error selecting thread."), "no_thread");	
		}
		
		$user = $runData->getUser();
		if($user == null){
			throw new WDPermissionException(_("Sorry, you must be logged in to add thread to watched."));	
		}
		
		$db = Database::connection();
		$db->begin();
		
		// check if you watch it already
		
		$c = new Criteria();
		$c->add("user_id", $user->getUserId());
		$c->add("thread_id", $threadId);
		
		$t = DB_WatchedForumThreadPeer::instance()->selectOne($c);
		
		if($t){
			throw new ProcessException(_("It seems you already watch this thread."), "already_watching");	
		}
		
		// ok, check how many do you already watch. 10 max ;-)
		$c = new Criteria();
		$c->add("user_id", $user->getUserId());
		
		$count = DB_WatchedForumThreadPeer::instance()->selectCount($c);
		if($count>9){
			throw new ProcessException(_("You can not watch more than 10 threads for now."), "max_reached");	
		}
		
		// ok, create new watch.
		
		$watch = new DB_WatchedForumThread();
		$watch->setUserId($user->getUserId());
		$watch->setThreadId($threadId);
		
		$watch->save();
		
		$db->commit();
		
	}
	
	public function removeWatchedThreadEvent($runData){
		$pl = $runData->getParameterList();
		
		$threadId = $pl->getParameterValue("threadId");
		
		if($threadId === null || !is_numeric($threadId)){
			throw new ProcessException(_("Can not process your request."));
		}
		
		$c = new Criteria();
		$c->add("thread_id", $threadId);
		$c->add("user_id", $runData->getUserId());
		
		DB_WatchedForumThreadPeer::instance()->delete($c);
			
	}
	
	public function watchPageEvent($runData){
		$pl = $runData->getParameterList();
		
		$pageId = $pl->getParameterValue('pageId');
		if($pageId === null || !is_numeric($pageId)){
			throw new ProcessException(_("Error selecting the page."), "no_page");	
		}
		
		$user = $runData->getUser();
		if($user == null){
			throw new WDPermissionException(_("Sorry, you must be logged in to add the page to watched."));	
		}
		
		$db = Database::connection();
		$db->begin();
		
		// check if you watch it already
		
		$c = new Criteria();
		$c->add("user_id", $user->getUserId());
		$c->add("page_id", $pageId);
		
		$t = DB_WatchedPagePeer::instance()->selectOne($c);
		
		if($t){
			throw new ProcessException(_("It seems you already watch this page."), "already_watching");	
		}
		
		// ok, check how many do you already watch. 10 max ;-)
		$c = new Criteria();
		$c->add("user_id", $user->getUserId());
		
		$count = DB_WatchedPagePeer::instance()->selectCount($c);
		if($count>9){
			throw new ProcessException(_("You can not watch more than 10 pages for now."), "max_reached");	
		}
		
		// ok, create new watch.
		
		$watch = new DB_WatchedPage();
		$watch->setUserId($user->getUserId());
		$watch->setPageId($pageId);
		
		$watch->save();
		
		$db->commit();
		
	}
	
	public function removeWatchedPageEvent($runData){
		$pl = $runData->getParameterList();
		
		$pageId = $pl->getParameterValue("pageId");
		
		if($pageId === null || !is_numeric($pageId)){
			throw new ProcessException(_("Can not process your request."));
		}
		
		$c = new Criteria();
		$c->add("page_id", $pageId);
		$c->add("user_id", $runData->getUserId());
		
		DB_WatchedPagePeer::instance()->delete($c);
			
	}
	
}
