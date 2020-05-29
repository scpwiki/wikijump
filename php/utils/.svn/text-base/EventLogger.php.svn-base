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

class EventLogger {
	
	private static $instance;
	
	public static function instance(){
		if(self::$instance == null){
			self::$instance = new EventLogger();
		}
		return self::$instance;
	}
	
	public function logLogin(){
	
		$e = $this->newEvent();
		$e->setType("LOGIN");
		$e->setText("User \"".htmlspecialchars($e->getTemp("user")->getNickName())."\" (".$e->getTemp("user")->getName().") logged in.");
		$e->save();
	
	}
	
	public function logFailedLogin($email){
		$e = $this->newEvent();
		$e->setType("FAILED_LOGIN");
		$e->setText("Failed login for username \"$email\"");
		$e->save();
	}

	public function logLogout(){
		$e = $this->newEvent();
		if($e->getTemp("user") == null){
			return;
		}
		$e->setType("LOGOUT");
		$e->setText("User \"".htmlspecialchars($e->getTemp("user")->getNickName())."\" (".$e->getTemp("user")->getName().") logged out.");
		$e->save();
	}
	
	public function logNewPage($page){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("PAGE_NEW");
		
		$e->setPageId($page->getPageId());
		$e->setRevisionId($page->getRevisionId());
		
		$e->setText('New page "'.htmlspecialchars($page->getUnixName()).'" has been saved on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}
	
	public function logSavePage($page){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("PAGE_EDIT");
		
		$e->setPageId($page->getPageId());
		$e->setRevisionId($page->getRevisionId());
		
		$e->setText('Page "'.htmlspecialchars($page->getUnixName()).'" has been (edited and) saved on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}
	
	public function logPageRename($page, $oldName){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("PAGE_RENAME");
		
		$e->setPageId($page->getPageId());
		$e->setRevisionId($page->getRevisionId());
		
		$e->setText('Page "'.htmlspecialchars($oldName).'" has been renamed to "'.htmlspecialchars($page->getUnixName()).'" on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}	
	public function logPageParentChange($page, $parentPage){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("PAGE_PARENT_CHANGE");
		
		$e->setPageId($page->getPageId());
		$e->setRevisionId($page->getRevisionId());
		
		$e->setText('Page "'.htmlspecialchars($page->getUnixName()).'" have a new parent: "'.($parentPage?htmlspecialchars($parentPage->getUnixName()):'').'" on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}	
	public function logNewThread($thread){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("THREAD_NEW");
		
		$e->setThreadId($thread->getThreadId());
		
		$e->setText('New thread "'.htmlspecialchars($thread->getTitle()).'" has been created on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}	
	public function logNewPost($post){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("POST_NEW");
		
		$e->setPostId($post->getPostId());
		$e->setThreadId($post->getThreadId());
		
		$e->setText('New post "'.htmlspecialchars($post->getTitle()).'" has been saved on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}	
	
	public function logSavePost($post){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("POST_SAVE");
		
		$e->setPostId($post->getPostId());
		$e->setThreadId($post->getThreadId());
		
		$e->setText('Post "'.htmlspecialchars($post->getTitle()).'" has been (edited and) saved on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}	
	public function logSaveThreadMeta($thread){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("THREAD_META_SAVE");
		
		$e->setThreadId($thread->getThreadId());
		
		$e->setText('Thread "'.htmlspecialchars($thread->getTitle()).'" meta has been changed on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}	
	
	public function logSaveThreadStickness($thread){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("THREAD_STICKNESS_SAVE");
		
		$e->setThreadId($thread->getThreadId());
		
		$e->setText('Thread "'.htmlspecialchars($thread->getTitle()).'" stickness been changed on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}	
	public function logSaveThreadBlock($thread){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("THREAD_BLOCK_SAVE");
		
		$e->setThreadId($thread->getThreadId());
		
		$e->setText('Thread "'.htmlspecialchars($thread->getTitle()).'" block been changed on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}	
	public function logThreadMoved($thread, $category){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("THREAD_MOVED");
		
		$e->setThreadId($thread->getThreadId());
		
		$e->setText('Thread "'.htmlspecialchars($thread->getTitle()).'" been moved to category "'.htmlspecialchars($category->getName()).'" on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}	
	
	public function logPostDelete($thread, $postTitle){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("POST_DELETED");
		
		$e->setThreadId($thread->getThreadId());
		
		$e->setText('Post "'.htmlspecialchars($postTitle).'" has been deleted in thread "'.htmlspecialchars($thread->getTitle()).'" on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();	
	}
	
	public function logFlagPage($page){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("ABUSE_PAGE_FLAG");
		
		$e->setText('Path "'.htmlspecialchars($page).'" has been flagged by user "'.htmlspecialchars($e->getTemp("user")->getNickName()).'" on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}
	
	public function logUnflagPage($page){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("ABUSE_PAGE_UNFLAG");
		
		$e->setText('Path "'.htmlspecialchars($page).'" has been unflagged by user "'.htmlspecialchars($e->getTemp("user")->getNickName()).'" on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}
	
	public function logFlagUser($user){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("ABUSE_USER_FLAG");
		
		$e->setText('User "'.htmlspecialchars($user->getNickName()).'" has been flagged by user "'.htmlspecialchars($e->getTemp("user")->getNickName()).'" on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}
	
	public function logUnflagUser($user){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("ABUSE_USER_UNFLAG");
		
		$e->setText('User "'.htmlspecialchars($user->getNickName()).'" has been unflagged by user "'.htmlspecialchars($e->getTemp("user")->getNickName()).'" on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}
	
	public function logFlagAnonymous($ipstring){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("ABUSE_ANONYMOUS_FLAG");
		
		$e->setText('Anonymous user "'.htmlspecialchars($ipstring).'" has been flagged by user "'.htmlspecialchars($e->getTemp("user")->getNickName()).'" on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}
	public function logUnflagAnonymous($ipstring){
		$e = $this->newEvent();
		$site = $e->getTemp("site");
		$e->setType("ABUSE_ANONYMOUS_UNFLAG");
		
		$e->setText('Anonymous user "'.htmlspecialchars($ipstring).'" has been unflagged by user "'.htmlspecialchars($e->getTemp("user")->getNickName()).'" on site "'.htmlspecialchars($site->getName()).'".');
		$e->save();
	}
	
	private function newEvent(){
		$event = new DB_LogEvent();
		$event->setDate(new ODate());
		
		// now to make things easier dig into some global variables and set what is needed
		$runData = OZONE::getRunData();
		
		//site
		
		$site = $runData->getTemp("site");
		
		$event->setSiteId($site->getSiteId());
		$event->setTemp("site", $site);
		
		// user_id (if any)
		$event->setUserId($runData->getUserId());
		$event->setTemp("user", $runData->getUser());
		
		//ip address
		
		list($ip, $proxy) = explode("|", $runData->createIpString());
		$event->setIp($ip);
		$event->setProxy($proxy);
		
		// user agent
		
		$event->setUserAgent($_SERVER['HTTP_USER_AGENT']);
		
		return $event;	
	}
	
}
