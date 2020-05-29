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
class DB_ForumPost extends DB_ForumPostBase {

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
	
	public function getEditedUser(){
		if($this->getEditedUserId() == 0){return null;}
		return DB_OzoneUserPeer::instance()->selectByPrimaryKey($this->getEditedUserId());
		
	}
	
	public function getEditedUserOrString(){
		$user = $this->getEditedUser();
		if($user == null){
			return $this->getEditedUserString();	
		}else{
			return $user;
		}
		
	}
	
	public function getForumThread(){
		if(is_array($this->prefetched)){
			if(in_array('forum_thread', $this->prefetched)){
				if(in_array('forum_thread', $this->prefetchedObjects)){
					return $this->prefetchedObjects['forum_thread'];
				} else {
					$obj = new DB_ForumThread($this->sourceRow, $this->prefetched);
					$obj->setNew(false);
					$this->prefetchedObjects['forum_thread'] = $obj;
					return $obj;
				}
			}
		}
		return DB_ForumThreadPeer::instance()->selectByPrimaryKey($this->getThreadId());
		
	}
	
	public function getSite(){
		if(is_array($this->prefetched)){
			if(in_array('site', $this->prefetched)){
				if(in_array('site', $this->prefetchedObjects)){
					return $this->prefetchedObjects['site'];
				} else {
					$obj = new DB_Site($this->sourceRow, $this->prefetched);
					$obj->setNew(false);
					$this->prefetchedObjects['site'] = $obj;
					return $obj;
				}
			}
		}
		return DB_SitePeer::instance()->selectByPrimaryKey($this->getSiteId());
		
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
	
	public function getRevision(){
		$r = DB_ForumPostRevisionPeer::instance()->selectByPrimaryKey($this->getRevisionId());
		return $r;	
	}
	
/*	
   	public function save(){
		$o = new Outdater();
		$o->forumEvent("post_save", $this);
		parent::save();	
	}
*/

	public function getPreview($length = 200){
		
		$text = $this->getText();
		$text =  preg_replace(';<table style=".*?id="toc".*?</table>;s', '', $text, 1);
		$stripped = strip_tags($text);
		$d = utf8_encode("\xFE");
		$stripped = preg_replace("/".$d."module \"([a-zA-Z0-9\/_]+?)\"(.+?)?".$d."/", '', $stripped);
		$stripped = str_replace($d, '', $stripped);
		// get last position of " "
		if(strlen8($stripped)>$length){
			$substr = substr($stripped, 0,$length);
			$length = strrpos($substr, " ");
			$substr = trim(substr($substr, 0,$length));
			$substr .= '...';
		}else{
			$substr = $stripped;	
		}
		return $substr;
	}
}
