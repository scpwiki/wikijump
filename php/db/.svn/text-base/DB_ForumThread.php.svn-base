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
 * @package Wikidot_Db
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * Object Model class.
 *
 */
class DB_ForumThread extends DB_ForumThreadBase {
	
	private $page;
	
	public function getUnixifiedTitle(){
		return WDStringUtils::toUnixName($this->getTitle());	
	}

	public function getTitle(){
		$pageId = $this->getPageId();
		if( $pageId == null){
			return parent::getTitle();
		}else{
			$page = $this->getPage();
			return $page->getTitle();
		}
				
	}
	
	public function getPage(){
		if($this->page){
			return $this->page;
		}else{
			if($this->getPageId() === null){return null;}
			$page = DB_PagePeer::instance()->selectByPrimaryKey($this->getPageId());
			$this->page = $page;
			return $page;	
		}	
	}
	
	public function getUser(){
		if($this->getUserId() == 0){return null;}
		if(is_array($this->prefetched)){
			if(in_array('ozone_user', $this->prefetched)){
				if(in_array('ozone_user', $this->prefetchedObjects)){
					return $this->prefetchedObjects['ozone_user'];
				} else {
					$obj = new DB_OzoneUser($this->sourceRow);
					$obj->setNew(false);
					$this->prefetchedObjects['ozone_user'] = $obj;
					return $obj;
				}
			}
		}
		return DB_OzoneUserPeer::instance()->selectByPrimaryKey($this->getUserId());
		
	}
	
	public function getUserOrString(){
		$user = $this->getUser();
		if($user == null){
			return $this->getUserString();	
		}else{
			return $user;
		}
		
	}
	
	public function getOzoneUser(){
		return $this->getUser();	
	}
	
	public function getLastPost(){
		if($this->getLastPostId() == null){
			return;
		}
		$c = new Criteria();
		$c->add("post_id", $this->getLastPostId());
		$c->addJoin("user_id", "ozone_user.user_id");
		
		$post = DB_ForumPostPeer::instance()->selectOne($c);
		return $post;	
	}
	
	/** 
	 * Scans for the last post.
	 */
	public function findLastPost(){
		$c = new Criteria();
		$c->add("thread_id", $this->getThreadId());
		$c->addOrderDescending("post_id");
		$post = DB_ForumPostPeer::instance()->selectOne($c);
		if($post){
			$this->setLastPostId($post->getPostId());	
		}
		return $post;
	}
	
	public function calculateNumberPosts(){
		$c = new Criteria();
		$c->add("thread_id", $this->getThreadId());
		$num = DB_ForumPostPeer::instance()->selectCount($c);
		$this->setNumberPosts($num);
	}
	
	public function getCategory(){
		$categoryId = $this->getCategoryId();

		$category = DB_ForumCategoryPeer::instance()->selectByPrimaryKey($categoryId);
		return $category;
			
	}
	
	public function getForumCategory(){
		if(is_array($this->prefetched)){
			if(in_array('forum_category', $this->prefetched)){
				if(in_array('forum_thread', $this->prefetchedObjects)){
					return $this->prefetchedObjects['forum_category'];
				} else {
					$obj = new DB_ForumCategory($this->sourceRow);
					$obj->setNew(false);
					$this->prefetchedObjects['forum_category'] = $obj;
					return $obj;
				}
			}
		}
		return DB_ForumCategoryPeer::instance()->selectByPrimaryKey($this->getCategoryId());
	}
	
	public function getFirstPost(){
		$c = new Criteria();
		$c->add("thread_id", $this->getThreadId());
		$c->addOrderAscending("post_id");
		$post = DB_ForumPostPeer::instance()->selectOne($c);
		return $post;	
	}
	
	public function getSite(){
		return DB_SitePeer::instance()->selectByPrimaryKey($this->getSiteId());	
	}
/*	
	public function save(){
		$o = new Outdater();
		$o->forumEvent("thread_save", $this);
		parent::save();	
	}
*/	
}
