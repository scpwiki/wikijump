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

class AWThreadsListModule extends AccountBaseModule {

	public function build($runData){
		
		$user = $runData->getUser();
		$runData->contextAdd("user",$user);	
		
		$pl = $runData->getParameterList();
		
		// get watched threads for this user
		
		$c = new Criteria();
		/*$c->add("watched_forum_thread.user_id", $user->getUserId());
		$c->addJoin("thread_id", "forum_thread.thread_id");
		$c->addOrderAscending("watched_id");
		*/
		/*
		$c->setExplicitFrom("forum_thread, watched_forum_thread");
		$c->add("watched_forum_thread.user_id", $user->getUserId());
		$c->
		*/
		
		$q = "SELECT forum_thread.* FROM watched_forum_thread, forum_thread " .
				"WHERE watched_forum_thread.user_id='".$user->getUserId()."' " .
						"AND watched_forum_thread.thread_id=forum_thread.thread_id";
		$c->setExplicitQuery($q);	
				
		$threads = DB_ForumThreadPeer::instance()->select($c);
		
		$runData->contextAdd("threads", $threads);
		
		$runData->contextAdd("threadsCount", count($threads));
		
	}

}
