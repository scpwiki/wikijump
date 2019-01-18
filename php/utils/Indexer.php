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

/** 
 * Full text search handler class.
 */
class Indexer {
	
	private static $instance;
	
	public static function instance(){
		if(self::$instance == null){
			self::$instance = new Indexer();
		}
		return 	self::$instance;
	}
	
	public function indexPage($page){
		// look for an existing fts_entry
		$ie = DB_FtsEntryPeer::instance()->selectByPageId($page->getPageId());
		if(!$ie){
			$ie = new DB_FtsEntry();
			$ie->setPageId($page->getPageId());
			$ie->setSiteId($page->getSiteId());	
		} 	
		// set properties (fields)
		$ie->setTitle(htmlspecialchars($page->getTitleOrUnixName()));
		$ie->setUnixName($page->getUnixName());
		
		$text = $page->getCompiled()->getText();
		$text = strip_tags($text);
		
		// kill modules
		$d = utf8_encode("\xFE");
		$text = preg_replace("/".$d."module \"([a-zA-Z0-9\/_]+?)\"([^".$d."]+?)?".$d."/", "\n", $text);
		$ie->setText($text);
		$title = db_escape_string(htmlspecialchars($page->getTitleOrUnixName()));
		$unixName =  db_escape_string(htmlspecialchars($page->getUnixName()));
		
		//get tags
		$c = new Criteria();
		$c->add("page_id", $page->getPageId());
		$c->addOrderAscending("tag");
		$tags = DB_PageTagPeer::instance()->select($c);
		$tagstring = '';
		foreach($tags as $tag){
			$tagstring .= $tag->getTag().' ';	
		}
		
	    $db = Database::connection();
    	$v = pg_version($db->getLink());
		if(!preg_match(';^8\.3;', $v['server'])){
		    $db->query("SELECT set_curcfg('default')");
		}
        $ie->setVector("(setweight( to_tsvector('$title'), 'A') || to_tsvector('".db_escape_string($text)."') || setweight( to_tsvector('$tagstring'), 'B'))", true);
		$ie->save();
	}
	
	public function deindexPage($page){
		$ie = DB_FtsEntryPeer::instance()->selectByPageId($page->getPageId());
		DB_FtsEntryPeer::instance()->deleteByPrimaryKey($ie->getFtsId());	
	}
	
	public function indexThread($thread){
		// look for an existing fts_entry
		$ie = DB_FtsEntryPeer::instance()->selectByThreadId($thread->getThreadId());
		if(!$ie){
			$ie = new DB_FtsEntry();
			$ie->setThreadId($thread->getThreadId());
			$ie->setSiteId($thread->getSiteId());
		}
		$ie->setTitle(htmlspecialchars($thread->getTitle()));
		$ie->setUnixName($thread->getUnixifiedTitle());
		// to create thread text select all posts and extract body
		
		$c = new Criteria();
		$c->add("thread_id", $thread->getThreadId());
		$c->addOrderAscending("post_id");
		$posts = DB_ForumPostPeer::instance()->select($c);
		
		$text = '';
		foreach($posts as $post){
			$text .= $post->getTitle()."\n";
			$text .= strip_tags($post->getText())."\n\n";
		}	
		$ie->setText(htmlspecialchars($thread->getDescription())."\n\n".$text);
		$title = db_escape_string(htmlspecialchars($thread->getTitle()));
		$description = db_escape_string(htmlspecialchars($thread->getDescription()));
		
	    $db = Database::connection();
    	$v = pg_version($db->getLink());
		if(!preg_match(';^8\.3;', $v['server'])){
		    $db->query("SELECT set_curcfg('default')");
		}
		
		$ie->setVector("setweight( to_tsvector('$title'), 'C') || setweight( to_tsvector('$description'), 'C') || to_tsvector('".db_escape_string($text)."')", true);
		
		$ie->save();
	}
	
	public function deindexThread($thread){
		$ie = DB_FtsEntryPeer::instance()->selectByThreadId($thread->getThreadId());
		DB_FtsEntryPeer::instance()->deleteByPrimaryKey($ie->getFtsId());		
	}
	
}
