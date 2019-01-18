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
 * This class searches for users given the query string and results in
 * an array of matches.
 */
class UserSearchModule extends SmartyModule {
	
	public function build($runData){
		$query = $runData->getParameterList()->getParameterValue("query");
		// split the query by ' '
		$q = explode(' ', $query);
		// escape regex syntax now
		for($i=0; $i<count($q); $i++){
			$q[$i] = preg_quote($q[$i], '/');	
		}
		$c = new Criteria();
		foreach($q as $q1){
			$c->add("nick_name", $q1, "~*");	
		}
		$c->setLimit(101);
		
		$users = DB_OzoneUserPeer::instance()->select($c);
		
		$runData->contextAdd("users", $users);
		
		// also prepare an array of user_id and nickname
		$runData->ajaxResponseAdd("count", count($users));
		if(count($users) == 101){
			$runData->ajaxResponseAdd("over100", true);	
		} else {
			$runData->ajaxResponseAdd("over100", false);	
		}
		
		$userIds = array();
		$userNames = array();
		foreach($users as $u){
			$userIds[] = $u->getUserId();	
			$userNames[$u->getUserId()] = htmlspecialchars($u->getNickName());
		}
		$runData->ajaxResponseAdd("userIds", $userIds);
		$runData->ajaxResponseAdd("userNames", $userNames);
	}
	
}
