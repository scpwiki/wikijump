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

class UserInfoWinModule extends SmartyModule {
	
	public function build($runData){
	
		$pl = $runData->getParameterList();
		$userId = $pl->getParameterValue("user_id");
		
		$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($userId);
		$avatarUri = '/common--images/avatars/'.floor($userId/1000).'/'.$userId.'/a48.png';
		$runData->contextAdd("user", $user); 
		$runData->contextAdd("avatarUri", $avatarUri);
		
		// find the possible role in this site
		
		$site = $runData->getTemp("site");
		$siteId = $site->getSiteId();
		
		$c = new Criteria();
		$c->add("user_id", $userId);
		$c->add("site_id", $siteId);
		$mem = DB_MemberPeer::instance()->selectOne($c);
		if( $mem != null){
			$runData->contextAdd("member", $mem);
			// also check for other roles: admin & moderator
			if(DB_AdminPeer::instance()->selectOne($c) != null){
				$runData->contextAdd("role", "admin");	
			}elseif(DB_AdminPeer::instance()->selectOne($c) != null){
				$runData->contextAdd("role", "moderator");	
			}
		}
		
		$runData->contextAdd("uu", $runData->getUser());
		$runData->contextAdd('karmaLevel', $user->getKarmaLevel());
		
	}
	
}
