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
class DB_ForumCategory extends DB_ForumCategoryBase {

	public function getUnixifiedName(){
		return WDStringUtils::toUnixName($this->getName());	
	}
	
	public function getEffectiveMaxNestLevel(){
		$nest = $this->getMaxNestLevel();
		if($nest == null){
			// get the value from forum settings
			$settings = DB_ForumSettingsPeer::instance()->selectByPrimaryKey($this->getSiteId());
			$nest = $settings->getMaxNestLevel();	
		}	
		return $nest;
	}
	
	public function calculateNumberPosts(){
		$q = "SELECT sum(number_posts) as posts FROM forum_thread WHERE category_id='".db_escape_string($this->getCategoryId())."'";
		$db = Database::connection();
		$r = $db->query($q);	
		$row = $r->nextRow();
		$n = $row['posts'];
		if($n === null){
			$n = 0;
		}
		$this->setNumberPosts($n);
	}
	
	public function calculateNumberThreads(){
		$c = new Criteria();
		$c->add("category_id", $this->getCategoryId());
		$num = DB_ForumThreadPeer::instance()->selectCount($c);
		$this->setNumberThreads($num);	
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
		$c->add("category_id", $this->getCategoryId());
		$c->add("last_post_id", null, "!=");
		$c->addOrderDescending("last_post_id");
		$thread = DB_ForumThreadPeer::instance()->selectOne($c);
		if($thread){
			$this->setLastPostId($thread->getLastPostId());	
		}else{
			$this->setLastPostId(null);
		}
	}
	
	public function getPermissionString(){
		if($this->getPermissions() == null || $this->getPermissions() == ''){
			$settings = DB_ForumSettingsPeer::instance()->selectByPrimaryKey($this->getSiteId());
			return $settings->getPermissions();	
		}else{
			return $this->getPermissions();	
		}
	}
	
	public function getForumGroup(){
		return DB_ForumGroupPeer::instance()->selectByPrimaryKey($this->getGroupId());	
	}
	
}
