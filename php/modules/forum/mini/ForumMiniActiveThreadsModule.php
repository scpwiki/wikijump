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

class ForumMiniActiveThreadsModule extends CacheableModule {
	
	protected $timeOut = 300;
	
	public function build($runData){
		
		$site = $runData->getTemp("site");
		
		// get recent forum threads
		$pl = $runData->getParameterList();
		$limit =  $pl->getParameterValue("limit", "MODULE");
		
		if($limit == null|| !is_numeric($limit) || $limit<1 || $limit>300){
			$limit = 5;	
		}
		
		$date = new ODate();
		$date->addSeconds(-60*60*24*7); // 7 days
		
		$q = "SELECT forum_thread.thread_id, count(*) AS count FROM forum_thread, forum_post " .
				"WHERE forum_thread.site_id='".$site->getSiteId()."' " .
				"AND forum_thread.thread_id = forum_post.thread_id " .
				"AND forum_post.date_posted > '". $date->getDate()."' " .
				"GROUP BY forum_thread.thread_id ORDER BY count DESC LIMIT ".db_escape_string($limit) ;
	
		$c = new Criteria();
		$c->setExplicitQuery($q);
		
		$threads = DB_ForumThreadPeer::instance()->select($c);
		
		foreach($threads as &$thread){
			$thread = DB_ForumThreadPeer::instance()->selectByPrimaryKey($thread->getThreadId());	
		}

		$runData->contextAdd("threads", $threads);

	}
	
}
