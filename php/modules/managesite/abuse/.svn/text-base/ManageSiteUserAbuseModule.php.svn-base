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

class ManageSiteUserAbuseModule extends ManageSiteBaseModule {
	
	public function build($runData){
		
		$site = $runData->getTemp("site");
		
		// get 
		$q = "SELECT target_user_id, count(*) AS rank " .
				"FROM user_abuse_flag " .
				"WHERE site_id='".$site->getSiteId()."' " .
				"AND site_valid = TRUE GROUP BY target_user_id ORDER BY rank DESC, target_user_id";

		$db = Database::connection();
		$res = $db->query($q);
		
		$all = $res->fetchAll();
		
		$r2 = array();
		
		if($all){
			foreach($all as &$r){
				// get user
				$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($r['target_user_id']);
				if($user){
					$r['user'] = $user;
					// check if member
					$c = new Criteria();
					$c->add("site_id", $site->getSiteId());
					$c->add("user_id", $user->getUserId());
					$mem = DB_MemberPeer::instance()->selectOne($c);
					if($mem){
						$r['member'] = $mem;	
					}
					$r2[] = $r;	
				}	
			}	
		}
		
		$runData->contextAdd("reps", $r2);

	}

}
